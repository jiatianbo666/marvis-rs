//! Task orchestrator — breaks complex tasks into sub-tasks and coordinates execution.

use marvis_core::MarvisError;

/// A sub-task that can be executed independently.
#[derive(Debug, Clone)]
pub struct SubTask {
    /// Human-readable description of this sub-task.
    pub description: String,
    /// The tool name to use (if any).
    pub tool_name: Option<String>,
    /// The tool arguments (if applicable).
    pub tool_args: Option<serde_json::Value>,
    /// Dependencies: indices of sub-tasks that must complete first.
    pub depends_on: Vec<usize>,
}

/// Result of executing a sub-task.
#[derive(Debug, Clone)]
pub struct SubTaskResult {
    /// Index of the original sub-task.
    pub index: usize,
    /// The result content.
    pub content: String,
    /// Whether this sub-task failed.
    pub is_error: bool,
}

/// The task orchestrator decomposes complex user requests and executes them.
pub struct TaskOrchestrator;

impl TaskOrchestrator {
    /// Create a new orchestrator.
    pub fn new() -> Self {
        Self
    }

    /// Decompose a user request into a sequence of sub-tasks.
    ///
    /// This is a simple keyword-based heuristic. In production, this would
    /// use the AI to plan the decomposition.
    pub fn decompose(&self, request: &str) -> Result<Vec<SubTask>, MarvisError> {
        let request_lower = request.to_lowercase();
        let mut tasks = Vec::new();

        // Heuristic: detect task types from keywords
        if (request_lower.contains("download") || request_lower.contains("整理"))
            && (request_lower.contains("下载") || request_lower.contains("download"))
        {
            tasks.push(SubTask {
                description: "List download directory contents".to_string(),
                tool_name: Some("list_directory".to_string()),
                tool_args: Some(serde_json::json!({
                    "path": ".",
                    "recursive": false,
                })),
                depends_on: vec![],
            });
        }

        if request_lower.contains("cpu")
            || request_lower.contains("进程")
            || request_lower.contains("process")
        {
            tasks.push(SubTask {
                description: "Get system process and CPU information".to_string(),
                tool_name: Some("list_processes".to_string()),
                tool_args: Some(serde_json::json!({
                    "sort_by": "cpu",
                    "limit": 10,
                })),
                depends_on: vec![],
            });
        }

        if request_lower.contains("搜索")
            || request_lower.contains("search")
            || request_lower.contains("文档")
        {
            tasks.push(SubTask {
                description: "Search for relevant information".to_string(),
                tool_name: Some("web_search".to_string()),
                tool_args: Some(serde_json::json!({
                    "query": request,
                    "limit": 5,
                })),
                depends_on: vec![],
            });
        }

        if request_lower.contains("文件")
            || request_lower.contains("file")
            || request_lower.contains("read")
        {
            tasks.push(SubTask {
                description: "Read and analyze files".to_string(),
                tool_name: Some("list_directory".to_string()),
                tool_args: Some(serde_json::json!({
                    "path": ".",
                    "recursive": false,
                })),
                depends_on: vec![],
            });
        }

        if tasks.is_empty() {
            // Default: get system overview
            tasks.push(SubTask {
                description: "Get system overview".to_string(),
                tool_name: Some("system_info".to_string()),
                tool_args: Some(serde_json::json!({})),
                depends_on: vec![],
            });
        }

        Ok(tasks)
    }

    /// Summarize the results of sub-task execution into a coherent response.
    pub fn summarize(&self, _request: &str, results: &[SubTaskResult]) -> String {
        if results.is_empty() {
            return "No sub-tasks were executed.".to_string();
        }

        let mut summary = String::from("📋 Task Execution Summary:\n\n");
        for result in results {
            let status = if result.is_error { "❌" } else { "✅" };
            summary.push_str(&format!(
                "{} Step {}: {}\n{}\n\n",
                status,
                result.index + 1,
                result.content.lines().next().unwrap_or("(empty)"),
                if result.content.lines().count() > 3 {
                    "... (see above for full output)"
                } else {
                    ""
                }
            ));
        }

        summary
    }
}

impl Default for TaskOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orchestrator() {
        let orch = TaskOrchestrator::new();
        // Should compile and not panic
    }

    #[test]
    fn test_decompose_cpu_request() {
        let orch = TaskOrchestrator::new();
        let tasks = orch.decompose("check CPU usage").unwrap();
        assert!(!tasks.is_empty());
        // Should detect CPU task
        assert!(tasks.iter().any(|t| t
            .tool_name
            .as_deref()
            .unwrap_or("")
            .contains("list_processes")));
    }

    #[test]
    fn test_decompose_download_request() {
        let orch = TaskOrchestrator::new();
        let tasks = orch.decompose("整理我的下载文件夹 download files").unwrap();
        assert!(!tasks.is_empty());
        assert!(tasks.iter().any(|t| t
            .tool_name
            .as_deref()
            .unwrap_or("")
            .contains("list_directory")));
    }

    #[test]
    fn test_decompose_default_fallback() {
        let orch = TaskOrchestrator::new();
        let tasks = orch.decompose("just a random question").unwrap();
        assert!(!tasks.is_empty());
        // Should fall back to system_info
        assert!(tasks
            .iter()
            .any(|t| t.tool_name.as_deref().unwrap_or("").contains("system_info")));
    }

    #[test]
    fn test_summarize() {
        let orch = TaskOrchestrator::new();
        let results = vec![
            SubTaskResult {
                index: 0,
                content: "OS: Windows\nArch: x86_64".to_string(),
                is_error: false,
            },
            SubTaskResult {
                index: 1,
                content: "Failed to fetch".to_string(),
                is_error: true,
            },
        ];
        let summary = orch.summarize("test", &results);
        assert!(summary.contains("✅"));
        assert!(summary.contains("❌"));
    }

    #[test]
    fn test_summarize_empty() {
        let orch = TaskOrchestrator::new();
        let summary = orch.summarize("test", &[]);
        assert!(summary.contains("No sub-tasks"));
    }
}
