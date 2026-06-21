//! Mock AI client for testing without a real API.

use async_trait::async_trait;
use marvis_core::{AiResponse, MarvisError, Message, StreamEvent, ToolCall, ToolSchema};

use crate::client::AiClient;

/// A mock AI client that returns preset responses.
/// Useful for testing and development without API keys.
pub struct MockClient {
    /// If set, always return this text response.
    pub text_response: Option<String>,
    /// If set, always return these tool calls.
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl MockClient {
    /// Create a new mock client that returns "Hello from mock!" by default.
    pub fn new() -> Self {
        Self {
            text_response: Some("Hello from mock AI! I am a simulated response.".to_string()),
            tool_calls: None,
        }
    }

    /// Create a mock client that returns specified tool calls.
    pub fn with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            text_response: Some("I'll use tools to help with that.".to_string()),
            tool_calls: Some(tool_calls),
        }
    }
}

impl Default for MockClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiClient for MockClient {
    async fn chat(
        &self,
        _messages: &[Message],
        _tools: &[ToolSchema],
    ) -> Result<AiResponse, MarvisError> {
        if let Some(ref calls) = self.tool_calls {
            Ok(AiResponse::ToolCalls(calls.clone()))
        } else {
            Ok(AiResponse::Text(
                self.text_response
                    .clone()
                    .unwrap_or_else(|| "Mock response".to_string()),
            ))
        }
    }

    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: &[ToolSchema],
    ) -> Result<Vec<StreamEvent>, MarvisError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use marvis_core::{Message, Role};

    #[test]
    fn test_mock_client_default_text() {
        let client = MockClient::new();
        assert!(client.text_response.is_some());
        assert!(client.tool_calls.is_none());
    }

    #[tokio::test]
    async fn test_mock_client_text_response() {
        let client = MockClient::new();
        let result = client.chat(&[], &[]).await.unwrap();
        match result {
            AiResponse::Text(text) => assert!(!text.is_empty()),
            _ => panic!("Expected Text response"),
        }
    }

    #[tokio::test]
    async fn test_mock_client_tool_calls() {
        let calls = vec![ToolCall {
            id: "call_1".to_string(),
            name: "read_file".to_string(),
            arguments: serde_json::json!({"path": "test.txt"}),
        }];
        let client = MockClient::with_tool_calls(calls.clone());
        let result = client.chat(&[], &[]).await.unwrap();
        match result {
            AiResponse::ToolCalls(tc) => {
                assert_eq!(tc.len(), 1);
                assert_eq!(tc[0].name, "read_file");
            }
            _ => panic!("Expected ToolCalls response"),
        }
    }

    #[tokio::test]
    async fn test_mock_client_stream_text() {
        let client = MockClient::new();
        let events = client.chat_stream(&[], &[]).await.unwrap();
        assert!(events.len() >= 2);
        assert!(matches!(events.first().unwrap(), StreamEvent::TextDelta(_)));
        assert!(matches!(events.last().unwrap(), StreamEvent::Done));
    }

    #[tokio::test]
    async fn test_mock_client_stream_tool_calls() {
        let calls = vec![ToolCall {
            id: "call_x".to_string(),
            name: "write_file".to_string(),
            arguments: serde_json::json!({"path": "out.txt"}),
        }];
        let client = MockClient::with_tool_calls(calls);
        let events = client.chat_stream(&[], &[]).await.unwrap();
        // Should have ToolCallStart, ToolCallEnd, Done
        assert!(events
            .iter()
            .any(|e| matches!(e, StreamEvent::ToolCallStart { .. })));
        assert!(events.iter().any(|e| matches!(e, StreamEvent::Done)));
    }
}
