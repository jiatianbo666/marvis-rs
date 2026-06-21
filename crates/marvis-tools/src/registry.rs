//! Tool registry for managing and dispatching tools.

use marvis_core::{MarvisError, Tool, ToolResult, ToolSchema};
use std::collections::HashMap;
use std::sync::Arc;

/// Central registry that stores all available tools and dispatches calls.
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool. Returns an error if a tool with the same name already exists.
    pub fn register(&mut self, tool: impl Tool + 'static) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// Execute a named tool with the given arguments.
    pub async fn execute(
        &self,
        name: &str,
        args: &serde_json::Value,
    ) -> Result<ToolResult, MarvisError> {
        let tool = self.get(name).ok_or_else(|| MarvisError::ToolError {
            tool: name.to_string(),
            message: format!("Tool '{}' not found", name),
        })?;
        tool.execute(args.clone()).await
    }

    /// Get schemas for all registered tools (for sending to the AI).
    pub fn schemas(&self) -> Vec<ToolSchema> {
        self.tools.values().map(|t| t.to_schema()).collect()
    }

    /// List all registered tool names.
    pub fn list_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a tool is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use marvis_core::RiskLevel;

    struct DummyTool {
        name: &'static str,
    }

    #[async_trait]
    impl Tool for DummyTool {
        fn name(&self) -> &str {
            self.name
        }
        fn description(&self) -> &str {
            "dummy"
        }
        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        async fn execute(&self, _input: serde_json::Value) -> Result<ToolResult, MarvisError> {
            Ok(ToolResult::success("dummy", "ok"))
        }
        fn risk_level(&self) -> RiskLevel {
            RiskLevel::ReadOnly
        }
    }

    #[test]
    fn test_new_registry_is_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = ToolRegistry::new();
        registry.register(DummyTool { name: "tool_a" });
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("tool_a"));
        assert!(registry.get("tool_a").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_register_multiple_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(DummyTool { name: "a" });
        registry.register(DummyTool { name: "b" });
        registry.register(DummyTool { name: "c" });
        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn test_list_names() {
        let mut registry = ToolRegistry::new();
        registry.register(DummyTool { name: "alpha" });
        registry.register(DummyTool { name: "beta" });
        let names = registry.list_names();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
    }

    #[test]
    fn test_schemas() {
        let mut registry = ToolRegistry::new();
        registry.register(DummyTool { name: "x" });
        registry.register(DummyTool { name: "y" });
        let schemas = registry.schemas();
        assert_eq!(schemas.len(), 2);
    }

    #[tokio::test]
    async fn test_execute_registered_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(DummyTool { name: "dummy" });
        let result = registry
            .execute("dummy", &serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(result.content, "ok");
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_execute_nonexistent_tool() {
        let registry = ToolRegistry::new();
        let result = registry.execute("ghost", &serde_json::json!({})).await;
        assert!(result.is_err());
    }
}
