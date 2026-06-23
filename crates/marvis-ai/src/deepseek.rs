//! DeepSeek API client implementation.
//!
//! Uses the OpenAI-compatible chat completions format.

use async_trait::async_trait;
use marvis_core::{AiResponse, MarvisError, Message, StreamEvent, ToolCall, ToolSchema};

use crate::client::AiClient;
use crate::types::*;

/// Client for the DeepSeek API (OpenAI-compatible format).
pub struct DeepSeekClient {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl DeepSeekClient {
    /// Create a new DeepSeek API client.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a new DeepSeek client with a custom base URL.
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

    /// Convert internal Message format to OpenAI-compatible ChatMessage.
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
                    tool_calls: if role == "assistant" {
                        m.tool_calls.as_ref().map(|tc| {
                            tc.iter()
                                .map(|tc| ToolCallDelta {
                                    id: tc.id.clone(),
                                    call_type: "function".to_string(),
                                    function: FunctionCall {
                                        name: tc.name.clone(),
                                        arguments: tc.arguments.to_string(),
                                    },
                                })
                                .collect()
                        })
                    } else {
                        None
                    },
                    tool_call_id: m.tool_call_id.clone(),
                    name: m.name.clone(),
                }
            })
            .collect()
    }

    /// Convert ToolSchema to OpenAI tool format.
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
impl AiClient for DeepSeekClient {
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
            tool_choice: if tools.is_empty() {
                None
            } else {
                Some("auto".to_string())
            },
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
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

        // Check for tool calls
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

        // Text response
        let content = choice.message.content.unwrap_or_default();
        Ok(AiResponse::Text(content))
    }

    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: &[ToolSchema],
    ) -> Result<Vec<StreamEvent>, MarvisError> {
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
            stream: Some(true),
            tool_choice: if tools.is_empty() {
                None
            } else {
                Some("auto".to_string())
            },
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| MarvisError::AiError(format!("HTTP request failed: {e}")))?;

        let mut events = Vec::new();
        let mut stream = response.bytes_stream();
        use futures::StreamExt;

        let mut buffer = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| MarvisError::AiError(format!("Stream error: {e}")))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process SSE lines
            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        events.push(StreamEvent::Done);
                        continue;
                    }

                    if let Ok(delta) = serde_json::from_str::<StreamDelta>(data) {
                        for choice in delta.choices {
                            if let Some(content) = choice.delta.content {
                                if !content.is_empty() {
                                    events.push(StreamEvent::TextDelta(content));
                                }
                            }
                            if let Some(tc_deltas) = choice.delta.tool_calls {
                                for tc in tc_deltas {
                                    let call_id = tc.id.clone();
                                    events.push(StreamEvent::ToolCallStart {
                                        id: call_id,
                                        name: tc.function.name,
                                    });
                                    events.push(StreamEvent::ToolCallEnd { id: tc.id });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(events)
    }
}
