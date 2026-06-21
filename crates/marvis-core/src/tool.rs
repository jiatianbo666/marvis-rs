use crate::{ToolResult, ToolSchema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Risk level of a tool operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Read-only operations — safe to run without confirmation
    ReadOnly = 0,
    /// Normal write operations — may need confirmation depending on mode
    Normal = 1,
    /// Dangerous operations — always require confirmation
    Dangerous = 2,
}

/// The `Tool` trait defines the interface that every tool must implement.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name used to identify this tool.
    fn name(&self) -> &str;

    /// Human-readable description shown to the AI.
    fn description(&self) -> &str;

    /// JSON Schema describing the expected input parameters.
    fn input_schema(&self) -> serde_json::Value;

    /// Execute the tool with the given JSON input.
    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, crate::MarvisError>;

    /// Whether this tool requires user confirmation before execution.
    /// Default: depends on `risk_level()`
    fn requires_confirmation(&self) -> bool {
        self.risk_level() == RiskLevel::Dangerous
    }

    /// The risk level of this tool.
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Normal
    }

    /// Build a ToolSchema for sending to the AI.
    fn to_schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.input_schema(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock tool for testing the Tool trait.
    struct MockTool {
        name: &'static str,
        desc: &'static str,
        risk: RiskLevel,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            self.name
        }

        fn description(&self) -> &str {
            self.desc
        }

        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object", "properties": {}})
        }

        async fn execute(
            &self,
            _input: serde_json::Value,
        ) -> Result<ToolResult, crate::MarvisError> {
            Ok(ToolResult::success("mock", "ok"))
        }

        fn risk_level(&self) -> RiskLevel {
            self.risk
        }
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Dangerous > RiskLevel::Normal);
        assert!(RiskLevel::Normal > RiskLevel::ReadOnly);
        assert_eq!(RiskLevel::ReadOnly as i32, 0);
        assert_eq!(RiskLevel::Dangerous as i32, 2);
    }

    #[test]
    fn test_tool_name_and_description() {
        let tool = MockTool {
            name: "test_tool",
            desc: "A test tool",
            risk: RiskLevel::ReadOnly,
        };
        assert_eq!(tool.name(), "test_tool");
        assert_eq!(tool.description(), "A test tool");
    }

    #[test]
    fn test_tool_requires_confirmation() {
        let dangerous = MockTool {
            name: "d",
            desc: "",
            risk: RiskLevel::Dangerous,
        };
        assert!(dangerous.requires_confirmation());

        let safe = MockTool {
            name: "s",
            desc: "",
            risk: RiskLevel::ReadOnly,
        };
        assert!(!safe.requires_confirmation());
    }

    #[test]
    fn test_tool_to_schema() {
        let tool = MockTool {
            name: "my_tool",
            desc: "Does things",
            risk: RiskLevel::Normal,
        };
        let schema = tool.to_schema();
        assert_eq!(schema.name, "my_tool");
        assert_eq!(schema.description, "Does things");
        assert!(schema.parameters.is_object());
    }

    #[tokio::test]
    async fn test_tool_execute() {
        let tool = MockTool {
            name: "t",
            desc: "",
            risk: RiskLevel::Normal,
        };
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert_eq!(result.content, "ok");
        assert!(!result.is_error);
    }
}
