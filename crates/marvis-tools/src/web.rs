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

/// Open a URL in the default browser.
pub struct OpenBrowser;

#[async_trait]
impl Tool for OpenBrowser {
    fn name(&self) -> &str {
        "open_browser"
    }

    fn description(&self) -> &str {
        "Open a URL in the system default browser. Use this to search the web or visit websites. \
         Provide either 'url' (full URL) or 'search_query' (opens Google search)."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to open (e.g. https://www.google.com/search?q=rust)"
                },
                "search_query": {
                    "type": "string",
                    "description": "Alternative: a search query. Will open Google search."
                }
            },
            "required": []
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Normal
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let url = if let Some(u) = input["url"].as_str() {
            u.to_string()
        } else if let Some(q) = input["search_query"].as_str() {
            format!(
                "https://www.google.com/search?q={}",
                q.chars()
                    .map(|c| match c {
                        ' ' => '+'.to_string(),
                        _ => c.to_string(),
                    })
                    .collect::<String>()
            )
        } else {
            return Err(MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Provide 'url' or 'search_query'".to_string(),
            });
        };

        open_with_shell(&url)
            .map(|()| ToolResult::success("open_browser", format!("Browser opened: {}", url)))
            .map_err(|e| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: e,
            })
    }
}

/// Platform-aware browser opening with fallbacks.
pub fn open_with_shell(url: &str) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        // Method 1: cmd /c start
        if std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
        // Method 2: explorer.exe
        if std::process::Command::new("explorer.exe")
            .arg(url)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
        // Method 3: rundll32
        if std::process::Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
        Err("Cannot open browser on Windows".into())
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed: {}", e))?;
        Ok(())
    } else {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed: {}", e))?;
        Ok(())
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

    #[tokio::test]
    async fn test_open_browser_valid_url() {
        let tool = OpenBrowser;
        let result = tool
            .execute(serde_json::json!({"url": "https://example.com"}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("Browser opened"));
    }

    #[tokio::test]
    async fn test_open_browser_search() {
        let tool = OpenBrowser;
        let result = tool
            .execute(serde_json::json!({"search_query": "rust lang"}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("Browser opened"));
    }

    #[test]
    fn test_tool_risk_levels() {
        assert_eq!(WebFetch.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(WebSearch.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(OpenBrowser.risk_level(), RiskLevel::Normal);
    }
}
