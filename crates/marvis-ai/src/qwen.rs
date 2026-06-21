//! Qwen Vision API client implementation via SiliconFlow.
//!
//! Uses the OpenAI-compatible chat completions format.

use async_trait::async_trait;
use marvis_core::{AiResponse, MarvisError, Message, StreamEvent, ToolCall, ToolSchema};

use crate::client::AiClient;
use crate::types::*;

/// Client for the Qwen Vision API via SiliconFlow.
pub struct QwenClient {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl QwenClient {
    /// Create a new Qwen API client.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            base_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a new Qwen client with a custom base URL.
    pub fn with_base_url(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<ChatMessage> {
        messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    marvis_core::Role::System => "system",
                    marvis_core::Role::User => "user",
                    marvis_core::Role::Assistant => "assistant",
                    marvis_core::Role::Tool => "tool",
                };
                ChatMessage {
                    role: role.to_string(),
                    content: Some(m.content.clone()),
                    tool_calls: None,
                    tool_call_id: m.tool_call_id.clone(),
                    name: m.name.clone(),
                }
            })
            .collect()
    }

    fn convert_tools(&self, tools: &[ToolSchema]) -> Vec<OpenAiTool> {
        tools
            .iter()
            .map(|t| OpenAiTool {
                tool_type: "function".to_string(),
                function: OpenAiFunction {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: t.parameters.clone(),
                },
            })
            .collect()
    }
}

#[async_trait]
impl AiClient for QwenClient {
    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolSchema],
    ) -> Result<AiResponse, MarvisError> {
        let chat_messages = self.convert_messages(messages);
        let openai_tools = if tools.is_empty() {
            None
        } else {
            Some(self.convert_tools(tools))
        };

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: chat_messages,
            tools: openai_tools,
            stream: Some(false),
            tool_choice: if tools.is_empty() { None } else { Some("auto".to_string()) },
        };

        let url = if self.base_url.ends_with("/chat/completions") {
            self.base_url.clone()
        } else {
            format!("{}/chat/completions", self.base_url)
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| MarvisError::AiError(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MarvisError::AiError(format!(
                "API returned {}: {}",
                status, body
            )));
        }

        let completion: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| MarvisError::AiError(format!("Failed to parse response: {e}")))?;

        let choice = completion
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| MarvisError::AiError("No choices in API response".to_string()))?;

        if let Some(tool_calls) = choice.message.tool_calls {
            if !tool_calls.is_empty() {
                let calls: Vec<ToolCall> = tool_calls
                    .into_iter()
                    .map(|tc| {
                        let id = tc.id.clone();
                        ToolCall {
                            id,
                            name: tc.function.name,
                            arguments: serde_json::from_str(&tc.function.arguments)
                                .unwrap_or(serde_json::Value::String(tc.function.arguments)),
                        }
                    })
                    .collect();
                return Ok(AiResponse::ToolCalls(calls));
            }
        }

        let content = choice.message.content.unwrap_or_default();
        Ok(AiResponse::Text(content))
    }

    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: &[ToolSchema],
    ) -> Result<Vec<StreamEvent>, MarvisError> {
        // Qwen via SiliconFlow also supports streaming — reuse non-streaming for now
        // In production, this would implement SSE streaming
        let result = self.chat(messages, tools).await?;

        match result {
            AiResponse::Text(text) => Ok(vec![StreamEvent::TextDelta(text), StreamEvent::Done]),
            AiResponse::ToolCalls(calls) => {
                let mut events = Vec::new();
                for call in &calls {
                    events.push(StreamEvent::ToolCallStart {
                        id: call.id.clone(),
                        name: call.name.clone(),
                    });
                    events.push(StreamEvent::ToolCallEnd {
                        id: call.id.clone(),
                    });
                }
                events.push(StreamEvent::Done);
                Ok(events)
            }
        }
    }
}
