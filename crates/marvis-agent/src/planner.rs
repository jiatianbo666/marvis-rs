//! Plan generator — creates execution plans for complex user requests.

/// A single step in an execution plan.
#[derive(Debug, Clone)]
pub struct PlanStep {
    /// Step number (1-based).
    pub number: usize,
    /// What this step does.
    pub action: String,
    /// The tool to use.
    pub tool: String,
    /// Arguments for the tool.
    pub args: serde_json::Value,
    /// Why this step is needed.
    pub reason: String,
}

/// An execution plan composed of ordered steps.
#[derive(Debug, Clone)]
pub struct Plan {
    /// Summary of what the plan accomplishes.
    pub goal: String,
    /// Ordered steps.
    pub steps: Vec<PlanStep>,
}

impl Plan {
    /// Create a new execution plan.
    pub fn new(goal: impl Into<String>, steps: Vec<PlanStep>) -> Self {
        Self {
            goal: goal.into(),
            steps,
        }
    }

    /// Get the number of steps.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Whether the plan is empty.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// Generates execution plans based on user requests.
pub struct Planner;

impl Planner {
    /// Create a new planner.
    pub fn new() -> Self {
        Self
    }

    /// Generate a plan for a user request using heuristics.
    ///
    /// In production, this would use the AI to generate the plan.
    pub fn generate_plan(
        &self,
        request: &str,
        available_tools: &[String],
    ) -> Result<Plan, marvis_core::MarvisError> {
        let request_lower = request.to_lowercase();
        let mut steps = Vec::new();
        let mut step_num = 0;

        // Heuristic plan generation based on keywords
        if request_lower.contains("download") || request_lower.contains("下载") {
            if available_tools.contains(&"list_directory".to_string()) {
                step_num += 1;
                steps.push(PlanStep {
                    number: step_num,
                    action: "List files in the current directory to find downloads".to_string(),
                    tool: "list_directory".to_string(),
                    args: serde_json::json!({
                        "path": ".",
                        "recursive": false,
                    }),
                    reason: "Need to see what files are present".to_string(),
                });
            }
            if available_tools.contains(&"file_info".to_string()) {
                step_num += 1;
                steps.push(PlanStep {
                    number: step_num,
                    action: "Get file information for sorting".to_string(),
                    tool: "file_info".to_string(),
                    args: serde_json::json!({"path": "."}),
                    reason: "Need file metadata to organize by type/date".to_string(),
                });
            }
        }

        if (request_lower.contains("system") || request_lower.contains("系统"))
            && available_tools.contains(&"system_info".to_string())
        {
            step_num += 1;
            steps.push(PlanStep {
                number: step_num,
                action: "Get system overview".to_string(),
                tool: "system_info".to_string(),
                args: serde_json::json!({}),
                reason: "User wants system information".to_string(),
            });
        }

        if (request_lower.contains("cpu")
            || request_lower.contains("memory")
            || request_lower.contains("内存"))
            && available_tools.contains(&"cpu_info".to_string())
        {
            step_num += 1;
            steps.push(PlanStep {
                number: step_num,
                action: "Get CPU and memory information".to_string(),
                tool: "cpu_info".to_string(),
                args: serde_json::json!({}),
                reason: "User wants resource usage information".to_string(),
            });
        }

        if (request_lower.contains("process") || request_lower.contains("进程"))
            && available_tools.contains(&"list_processes".to_string())
        {
            step_num += 1;
            steps.push(PlanStep {
                number: step_num,
                action: "List top processes by CPU usage".to_string(),
                tool: "list_processes".to_string(),
                args: serde_json::json!({
                    "sort_by": "cpu",
                    "limit": 10,
                }),
                reason: "User wants to see process information".to_string(),
            });
        }

        if (request_lower.contains("web")
            || request_lower.contains("网页")
            || request_lower.contains("fetch"))
            && available_tools.contains(&"web_fetch".to_string())
        {
            step_num += 1;
            steps.push(PlanStep {
                number: step_num,
                action: "Fetch web content".to_string(),
                tool: "web_fetch".to_string(),
                args: serde_json::json!({"url": ""}),
                reason: "User wants web content".to_string(),
            });
        }

        if steps.is_empty() {
            // Default fallback plan
            if available_tools.contains(&"system_info".to_string()) {
                step_num += 1;
                steps.push(PlanStep {
                    number: step_num,
                    action: "Get system overview".to_string(),
                    tool: "system_info".to_string(),
                    args: serde_json::json!({}),
                    reason: "Default: provide system context".to_string(),
                });
            }
        }

        let goal = format!("Execute plan for: {}", request);
        Ok(Plan::new(goal, steps))
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_planner() {
        let planner = Planner::new();
    }

    #[test]
    fn test_generate_plan_system_request() {
        let planner = Planner::new();
        let tools = vec!["system_info".to_string(), "cpu_info".to_string()];
        let plan = planner
            .generate_plan("show me system information", &tools)
            .unwrap();
        assert!(!plan.is_empty());
        assert!(plan.steps.iter().any(|s| s.tool == "system_info"));
    }

    #[test]
    fn test_generate_plan_cpu_request() {
        let planner = Planner::new();
        let tools = vec!["cpu_info".to_string(), "memory_info".to_string()];
        let plan = planner
            .generate_plan("what's my CPU usage", &tools)
            .unwrap();
        assert!(!plan.is_empty());
        assert!(plan.steps.iter().any(|s| s.tool == "cpu_info"));
    }

    #[test]
    fn test_generate_plan_process_request() {
        let planner = Planner::new();
        let tools = vec!["list_processes".to_string()];
        let plan = planner
            .generate_plan("show running processes", &tools)
            .unwrap();
        assert!(!plan.is_empty());
        assert!(plan.steps.iter().any(|s| s.tool == "list_processes"));
    }

    #[test]
    fn test_generate_plan_fallback() {
        let planner = Planner::new();
        let tools = vec!["system_info".to_string()];
        let plan = planner.generate_plan("random thought", &tools).unwrap();
        assert!(!plan.is_empty());
    }

    #[test]
    fn test_plan_construction() {
        let steps = vec![PlanStep {
            number: 1,
            action: "test".to_string(),
            tool: "test_tool".to_string(),
            args: serde_json::json!({}),
            reason: "testing".to_string(),
        }];
        let plan = Plan::new("test goal", steps);
        assert_eq!(plan.len(), 1);
        assert!(!plan.is_empty());
    }
}
