//! The main agent loop — the core AI ↔ Tool interaction cycle.

use log::{debug, info};
use marvis_ai::AiClient;
use marvis_core::{AiResponse, MarvisError, StreamEvent, ToolCall};
use marvis_security::SecurityManager;
use marvis_session::ConversationHistory;
use marvis_tools::ToolRegistry;

/// Configuration for the agent loop.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Maximum number of tool-call iterations before giving up.
    pub max_iterations: usize,
    /// Whether to stream responses token-by-token.
    pub streaming: bool,
    /// Callback for streaming text output.
    pub on_text: Option<fn(&str)>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            streaming: true,
            on_text: None,
        }
    }
}

/// The main agent loop that processes user input and coordinates AI ↔ Tool interaction.
pub struct AgentLoop<'a> {
    client: &'a dyn AiClient,
    tools: &'a ToolRegistry,
    security: &'a SecurityManager,
    config: AgentConfig,
}

impl<'a> AgentLoop<'a> {
    /// Create a new agent loop.
    pub fn new(
        client: &'a dyn AiClient,
        tools: &'a ToolRegistry,
        security: &'a SecurityManager,
    ) -> Self {
        Self {
            client,
            tools,
            security,
            config: AgentConfig::default(),
        }
    }

    /// Create a new agent loop with custom config.
    pub fn with_config(
        client: &'a dyn AiClient,
        tools: &'a ToolRegistry,
        security: &'a SecurityManager,
        config: AgentConfig,
    ) -> Self {
        Self {
            client,
            tools,
            security,
            config,
        }
    }

    /// Run the agent loop for a single user input.
    ///
    /// Returns the final text response after all tool calls are resolved.
    pub async fn run(
        &self,
        history: &mut ConversationHistory,
        user_input: &str,
    ) -> Result<String, MarvisError> {
        // Add user message to history
        history.add_user(user_input);

        let mut iteration = 0;
        loop {
            iteration += 1;
            if iteration > self.config.max_iterations {
                return Err(MarvisError::MaxIterationsExceeded(
                    self.config.max_iterations,
                ));
            }

            debug!(
                "Agent loop iteration {}/{}",
                iteration, self.config.max_iterations
            );

            // Build messages for the AI
            let messages = history.messages().to_vec();
            let tool_schemas = self.tools.schemas();

            // Call AI
            let response = if self.config.streaming {
                let events = self.client.chat_stream(&messages, &tool_schemas).await?;
                self.process_stream_events(events)?
            } else {
                self.client.chat(&messages, &tool_schemas).await?
            };

            match response {
                AiResponse::Text(text) => {
                    history.add_assistant(&text, None);
                    info!(
                        "Agent loop complete after {} iterations (text response)",
                        iteration
                    );
                    return Ok(text);
                }
                AiResponse::ToolCalls(tool_calls) => {
                    info!(
                        "Agent received {} tool call(s) on iteration {}",
                        tool_calls.len(),
                        iteration
                    );

                    // Record the tool calls in history
                    history.add_assistant("", Some(tool_calls.clone()));

                    // Execute each tool call
                    for call in &tool_calls {
                        let result = self.execute_tool(call).await?;
                        history.add_tool_result(
                            &call.id,
                            result.content.clone(),
                            Some(call.name.clone()),
                        );
                    }

                    // Loop back to AI with tool results
                }
            }
        }
    }

    /// Execute a single tool call with security checks.
    async fn execute_tool(&self, call: &ToolCall) -> Result<marvis_core::ToolResult, MarvisError> {
        info!("Executing tool: {} (id: {})", call.name, call.id);

        // Security check
        self.security.check(&call.name, &call.arguments)?;

        // Check if confirmation is needed
        if self
            .security
            .needs_confirmation(&call.name, &call.arguments)
        {
            return Err(MarvisError::PermissionDenied {
                tool: call.name.clone(),
                reason: format!(
                    "Tool '{}' requires user confirmation. Please approve this operation.",
                    call.name
                ),
            });
        }

        // Execute the tool
        let result = self.tools.execute(&call.name, &call.arguments).await;

        match &result {
            Ok(r) => {
                if r.is_error {
                    info!("Tool '{}' returned an error: {}", call.name, r.content);
                } else {
                    info!("Tool '{}' executed successfully", call.name);
                }
            }
            Err(e) => {
                info!("Tool '{}' failed: {}", call.name, e);
            }
        }

        // If tool execution failed, return a tool result with is_error=true
        result.or_else(|e| {
            Ok(marvis_core::ToolResult::error(
                &call.id,
                format!("Tool execution failed: {}", e),
            ))
        })
    }

    /// Process stream events into an AiResponse.
    fn process_stream_events(&self, events: Vec<StreamEvent>) -> Result<AiResponse, MarvisError> {
        let mut text = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut current_tool: Option<(String, String, String)> = None; // (id, name, args_string)

        for event in &events {
            match event {
                StreamEvent::TextDelta(delta) => {
                    text.push_str(delta);
                    if let Some(callback) = self.config.on_text {
                        callback(delta);
                    }
                }
                StreamEvent::ToolCallStart { id, name } => {
                    current_tool = Some((id.clone(), name.clone(), String::new()));
                }
                StreamEvent::ToolCallDelta {
                    id: _id,
                    args_delta,
                } => {
                    if let Some((_id, _name, args_buf)) = current_tool.as_mut() {
                        args_buf.push_str(args_delta);
                    }
                }
                StreamEvent::ToolCallEnd { id: _end_id } => {
                    if let Some((id, name, args_str)) = current_tool.take() {
                        let arguments = if args_str.is_empty() {
                            serde_json::Value::Object(serde_json::Map::new())
                        } else if let Ok(val) = serde_json::from_str(&args_str) {
                            val
                        } else {
                            serde_json::Value::String(args_str)
                        };
                        tool_calls.push(ToolCall {
                            id,
                            name,
                            arguments,
                        });
                    }
                }
                StreamEvent::Done => {}
            }
        }

        if !tool_calls.is_empty() {
            Ok(AiResponse::ToolCalls(tool_calls))
        } else {
            Ok(AiResponse::Text(text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use marvis_ai::mock::MockClient;
    use marvis_security::{permissions::PermissionMode, SecurityManager};
    use marvis_session::ConversationHistory;
    use marvis_tools::ToolRegistry;

    fn setup() -> (MockClient, ToolRegistry, SecurityManager) {
        let client = MockClient::new();
        let registry = ToolRegistry::new();
        let security = SecurityManager::new(PermissionMode::Normal);
        (client, registry, security)
    }

    #[tokio::test]
    async fn test_agent_loop_text_response() {
        let (client, registry, security) = setup();
        let agent = AgentLoop::new(&client, &registry, &security);
        let mut history = ConversationHistory::new();

        let result = agent.run(&mut history, "hello").await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
        assert!(history.len() >= 2); // user + assistant
    }

    #[tokio::test]
    async fn test_agent_loop_multi_turn_tool_calls() {
        // The mock with tool_calls will always return tool_calls,
        // triggering the agent loop until max_iterations is hit.
        // This test verifies that the agent correctly processes tool calls
        // and handles the max iteration limit.
        let calls = vec![marvis_core::ToolCall {
            id: "t1".to_string(),
            name: "system_info".to_string(),
            arguments: serde_json::json!({}),
        }];
        let client = MockClient::with_tool_calls(calls);

        let mut registry = ToolRegistry::new();
        use async_trait::async_trait;
        use marvis_core::{RiskLevel, Tool, ToolResult as TR};
        struct DummyTool;
        #[async_trait]
        impl Tool for DummyTool {
            fn name(&self) -> &str {
                "system_info"
            }
            fn description(&self) -> &str {
                "dummy"
            }
            fn input_schema(&self) -> serde_json::Value {
                serde_json::json!({})
            }
            async fn execute(&self, _input: serde_json::Value) -> Result<TR, MarvisError> {
                Ok(TR::success("system_info", "OS: Windows"))
            }
            fn risk_level(&self) -> RiskLevel {
                RiskLevel::ReadOnly
            }
        }
        registry.register(DummyTool);

        let mut config = AgentConfig::default();
        config.max_iterations = 3;
        let security = SecurityManager::new(PermissionMode::Normal);
        let agent = AgentLoop::with_config(&client, &registry, &security, config);
        let mut history = ConversationHistory::new();

        let result = agent.run(&mut history, "check system").await;
        // Should exceed max iterations since mock always returns tool calls
        assert!(result.is_err());
        match result {
            Err(MarvisError::MaxIterationsExceeded(n)) => assert_eq!(n, 3),
            _ => panic!("Expected MaxIterationsExceeded"),
        }
        // History should have accumulated messages from each iteration
        assert!(history.len() > 2);
    }

    #[tokio::test]
    async fn test_agent_loop_max_iterations() {
        // Create a client that always returns tool calls → infinite loop
        let calls = vec![marvis_core::ToolCall {
            id: "t1".to_string(),
            name: "system_info".to_string(),
            arguments: serde_json::json!({}),
        }];
        let client = MockClient::with_tool_calls(calls);

        let mut registry = ToolRegistry::new();
        // Register a dummy tool
        use async_trait::async_trait;
        use marvis_core::{RiskLevel, Tool, ToolResult as TR};
        struct DummyTool;
        #[async_trait]
        impl Tool for DummyTool {
            fn name(&self) -> &str {
                "system_info"
            }
            fn description(&self) -> &str {
                "dummy"
            }
            fn input_schema(&self) -> serde_json::Value {
                serde_json::json!({})
            }
            async fn execute(&self, _input: serde_json::Value) -> Result<TR, MarvisError> {
                Ok(TR::success("system_info", "ok"))
            }
            fn risk_level(&self) -> RiskLevel {
                RiskLevel::ReadOnly
            }
        }
        registry.register(DummyTool);

        let mut config = AgentConfig::default();
        config.max_iterations = 3;
        let security = SecurityManager::new(PermissionMode::Normal);
        let agent = AgentLoop::with_config(&client, &registry, &security, config);
        let mut history = ConversationHistory::new();

        let result = agent.run(&mut history, "test").await;
        assert!(result.is_err());
        match result {
            Err(MarvisError::MaxIterationsExceeded(n)) => assert_eq!(n, 3),
            _ => panic!("Expected MaxIterationsExceeded"),
        }
    }
}
