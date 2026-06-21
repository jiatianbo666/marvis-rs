//! Web interaction tools.

use async_trait::async_trait;
use marvis_core::{MarvisError, RiskLevel, Tool, ToolResult};

/// Fetch content from a URL and extract text.
pub struct WebFetch;

#[async_trait]
impl Tool for WebFetch {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch content from a URL and extract the main text content."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch content from"
                }
            },
            "required": ["url"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::ReadOnly
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let url_str = input["url"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'url' parameter".to_string(),
            })?;

        let client = reqwest::Client::builder()
            .user_agent("marvis/0.1 (rust-ai-assistant)")
            .build()
            .map_err(|e| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        let response = client
            .get(url_str)
            .send()
            .await
            .map_err(|e| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: format!("Failed to fetch URL: {}", e),
            })?;

        let status = response.status();
        if !status.is_success() {
            return Ok(ToolResult::error(
                "web_fetch",
                format!("HTTP {} when fetching '{}'", status, url_str),
            ));
        }

        let html = response.text().await.map_err(|e| MarvisError::ToolError {
            tool: self.name().to_string(),
            message: format!("Failed to read response body: {}", e),
        })?;

        // Extract text from HTML
        let document = scraper::Html::parse_document(&html);

        // Remove script and style elements
        let text = document
            .root_element()
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(5000)
            .collect::<String>();

        if text.len() == 5000 {
            Ok(ToolResult::success(
                "web_fetch",
                format!("{}... [truncated to 5000 chars]", text),
            ))
        } else {
            Ok(ToolResult::success("web_fetch", text))
        }
    }
}

/// Search the web (delegates to a search engine).
pub struct WebSearch;

#[async_trait]
impl Tool for WebSearch {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information. Returns a list of relevant results."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default 5)"
                }
            },
            "required": ["query"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::ReadOnly
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let query = input["query"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'query' parameter".to_string(),
            })?;
        let _limit = input["limit"].as_u64().unwrap_or(5) as usize;

        // For now, return a message indicating this is a mock implementation.
        // In production, this would integrate with a real search API.
        Ok(ToolResult::success(
            "web_search",
            format!(
                "Web search for '{}' — this feature requires a search API integration.\n\
                 Suggested: Try using web_fetch with a search engine URL like \
                 'https://www.google.com/search?q={}'",
                query,
                query.replace(' ', "+")
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_fetch_missing_url() {
        let tool = WebFetch;
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_web_fetch_invalid_url() {
        let tool = WebFetch;
        let result = tool
            .execute(serde_json::json!({"url": "not-a-valid-url"}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_web_search_basic() {
        let tool = WebSearch;
        let result = tool
            .execute(serde_json::json!({"query": "rust programming"}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("rust"));
    }

    #[tokio::test]
    async fn test_web_search_missing_query() {
        let tool = WebSearch;
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_risk_levels() {
        assert_eq!(WebFetch.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(WebSearch.risk_level(), RiskLevel::ReadOnly);
    }
}
