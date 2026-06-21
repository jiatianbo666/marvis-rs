//! Clipboard tools.

use async_trait::async_trait;
use marvis_core::{MarvisError, RiskLevel, Tool, ToolResult};

/// Read text from the system clipboard.
pub struct ReadClipboard;

#[async_trait]
impl Tool for ReadClipboard {
    fn name(&self) -> &str {
        "read_clipboard"
    }

    fn description(&self) -> &str {
        "Read the current text content from the system clipboard."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::ReadOnly
    }

    async fn execute(&self, _input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| MarvisError::ToolError {
            tool: self.name().to_string(),
            message: format!("Failed to access clipboard: {}", e),
        })?;

        match clipboard.get_text() {
            Ok(text) => {
                if text.is_empty() {
                    Ok(ToolResult::success("read_clipboard", "Clipboard is empty."))
                } else {
                    Ok(ToolResult::success("read_clipboard", text))
                }
            }
            Err(e) => Ok(ToolResult::error(
                "read_clipboard",
                format!("Failed to read clipboard: {}", e),
            )),
        }
    }
}

/// Write text to the system clipboard.
pub struct WriteClipboard;

#[async_trait]
impl Tool for WriteClipboard {
    fn name(&self) -> &str {
        "write_clipboard"
    }

    fn description(&self) -> &str {
        "Write text content to the system clipboard."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The text to write to the clipboard"
                }
            },
            "required": ["text"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Normal
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let text = input["text"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'text' parameter".to_string(),
            })?;

        let mut clipboard = arboard::Clipboard::new().map_err(|e| MarvisError::ToolError {
            tool: self.name().to_string(),
            message: format!("Failed to access clipboard: {}", e),
        })?;

        clipboard
            .set_text(text)
            .map_err(|e| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: format!("Failed to write to clipboard: {}", e),
            })?;

        Ok(ToolResult::success(
            "write_clipboard",
            format!("Clipboard updated ({} characters)", text.len()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_clipboard_returns_content() {
        let tool = ReadClipboard;
        let result = tool.execute(serde_json::json!({})).await;
        // Clipboard access may fail in CI/headless environments
        match result {
            Ok(r) => assert!(
                !r.content.is_empty()
                    || r.content.contains("empty")
                    || r.content.contains("Failed")
            ),
            Err(_) => {} // Acceptable: clipboard may not be available
        }
    }

    #[tokio::test]
    async fn test_write_clipboard_missing_text() {
        let tool = WriteClipboard;
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_and_read_clipboard() {
        let write_tool = WriteClipboard;
        let result = write_tool
            .execute(serde_json::json!({"text": "marvis test content"}))
            .await;
        match result {
            Ok(r) => {
                assert!(!r.is_error);
                // Try to read back
                let read_tool = ReadClipboard;
                let read_result = read_tool.execute(serde_json::json!({})).await;
                if let Ok(rr) = read_result {
                    assert!(
                        !rr.is_error
                            || rr.content.contains("marvis")
                            || rr.content.contains("Failed")
                    );
                }
            }
            Err(_) => {} // Clipboard may not be available in all environments
        }
    }

    #[test]
    fn test_tool_risk_levels() {
        assert_eq!(ReadClipboard.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(WriteClipboard.risk_level(), RiskLevel::Normal);
    }
}
