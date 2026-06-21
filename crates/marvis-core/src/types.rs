use serde::{Deserialize, Serialize};

/// Represents a role in a chat conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    /// Tool call ID for tool results, or tool calls embedded in assistant messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Name of the tool that produced this result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    /// Create a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            tool_call_id: None,
            tool_calls: None,
            name: None,
        }
    }

    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            tool_call_id: None,
            tool_calls: None,
            name: None,
        }
    }

    /// Create a new assistant message, optionally with tool calls.
    pub fn assistant(content: impl Into<String>, tool_calls: Option<Vec<ToolCall>>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            tool_call_id: None,
            tool_calls,
            name: None,
        }
    }

    /// Create a tool result message.
    pub fn tool_result(
        tool_call_id: impl Into<String>,
        content: impl Into<String>,
        tool_name: Option<String>,
    ) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
            tool_calls: None,
            name: tool_name,
        }
    }
}

/// A tool call requested by the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    /// JSON-encoded arguments
    pub arguments: serde_json::Value,
}

/// Result of executing a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

impl ToolResult {
    /// Create a successful tool result.
    pub fn success(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    /// Create an error tool result.
    pub fn error(tool_call_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: message.into(),
            is_error: true,
        }
    }
}

/// Schema describing a tool to the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    /// JSON Schema for the tool's input parameters
    pub parameters: serde_json::Value,
}

/// Response from the AI — either text or tool calls.
#[derive(Debug, Clone)]
pub enum AiResponse {
    /// Plain text response
    Text(String),
    /// The AI wants to call one or more tools
    ToolCalls(Vec<ToolCall>),
}

/// Events emitted during streaming responses.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// A chunk of text content
    TextDelta(String),
    /// Start of a tool call
    ToolCallStart { id: String, name: String },
    /// Streaming arguments delta for a tool call
    ToolCallDelta { id: String, args_delta: String },
    /// Tool call arguments complete
    ToolCallEnd { id: String },
    /// Stream is done
    Done,
}

/// Session configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub max_history_tokens: usize,
    pub save_on_exit: bool,
    pub storage_dir: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_history_tokens: 8000,
            save_on_exit: true,
            storage_dir: ".marvis/sessions".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation_user() {
        let msg = Message::user("hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "hello");
        assert!(msg.tool_call_id.is_none());
    }

    #[test]
    fn test_message_creation_system() {
        let msg = Message::system("You are helpful");
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.content, "You are helpful");
    }

    #[test]
    fn test_message_creation_assistant_with_tool_calls() {
        let calls = vec![ToolCall {
            id: "call_1".to_string(),
            name: "read_file".to_string(),
            arguments: serde_json::json!({"path": "test.txt"}),
        }];
        let msg = Message::assistant("I'll read that file", Some(calls.clone()));
        assert_eq!(msg.role, Role::Assistant);
        assert!(msg.tool_calls.is_some());
        assert_eq!(msg.tool_calls.unwrap().len(), 1);
    }

    #[test]
    fn test_message_creation_tool_result() {
        let msg = Message::tool_result(
            "call_1",
            "file contents here",
            Some("read_file".to_string()),
        );
        assert_eq!(msg.role, Role::Tool);
        assert_eq!(msg.tool_call_id, Some("call_1".to_string()));
        assert_eq!(msg.name, Some("read_file".to_string()));
    }

    #[test]
    fn test_tool_call_serialization() {
        let call = ToolCall {
            id: "abc123".to_string(),
            name: "read_file".to_string(),
            arguments: serde_json::json!({"path": "/tmp/test.txt"}),
        };
        let json = serde_json::to_string(&call).unwrap();
        let parsed: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "abc123");
        assert_eq!(parsed.name, "read_file");
        assert_eq!(parsed.arguments["path"], "/tmp/test.txt");
    }

    #[test]
    fn test_tool_result_is_error() {
        let success = ToolResult::success("id1", "all good");
        assert!(!success.is_error);

        let error = ToolResult::error("id2", "something went wrong");
        assert!(error.is_error);
    }

    #[test]
    fn test_tool_result_content() {
        let result = ToolResult::success("call_x", "hello world");
        assert_eq!(result.tool_call_id, "call_x");
        assert_eq!(result.content, "hello world");
        assert!(!result.is_error);
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.max_history_tokens, 8000);
        assert!(config.save_on_exit);
        assert_eq!(config.storage_dir, ".marvis/sessions");
    }

    #[test]
    fn test_ai_response_variants() {
        let text = AiResponse::Text("hello".to_string());
        match text {
            AiResponse::Text(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected Text variant"),
        }

        let calls = AiResponse::ToolCalls(vec![]);
        match calls {
            AiResponse::ToolCalls(v) => assert!(v.is_empty()),
            _ => panic!("Expected ToolCalls variant"),
        }
    }

    #[test]
    fn test_tool_schema_construction() {
        let schema = ToolSchema {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({"type": "object"}),
        };
        assert_eq!(schema.name, "test_tool");
        assert_eq!(schema.description, "A test tool");
        assert!(schema.parameters.is_object());
    }

    #[test]
    fn test_stream_event_variants() {
        let delta = StreamEvent::TextDelta("hi".to_string());
        let start = StreamEvent::ToolCallStart {
            id: "1".to_string(),
            name: "t".to_string(),
        };
        let end = StreamEvent::ToolCallEnd {
            id: "1".to_string(),
        };
        let done = StreamEvent::Done;

        assert!(matches!(delta, StreamEvent::TextDelta(_)));
        assert!(matches!(start, StreamEvent::ToolCallStart { .. }));
        assert!(matches!(end, StreamEvent::ToolCallEnd { .. }));
        assert!(matches!(done, StreamEvent::Done));
    }
}
