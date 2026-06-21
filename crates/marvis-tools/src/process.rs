//! Process management tools.

use async_trait::async_trait;
use marvis_core::{MarvisError, RiskLevel, Tool, ToolResult};

/// List running processes.
pub struct ListProcesses;

#[async_trait]
impl Tool for ListProcesses {
    fn name(&self) -> &str {
        "list_processes"
    }

    fn description(&self) -> &str {
        "List running processes on the system, optionally sorted by CPU or memory usage."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "sort_by": {
                    "type": "string",
                    "enum": ["cpu", "memory", "name"],
                    "description": "Sort processes by this field"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of processes to return (default 20)"
                }
            },
            "required": []
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::ReadOnly
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let limit = input["limit"].as_u64().unwrap_or(20) as usize;
        let sort_by = input["sort_by"].as_str().unwrap_or("cpu");

        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let mut processes: Vec<_> = sys.processes().iter().collect();

        match sort_by {
            "cpu" => {
                processes.sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap())
            }
            "memory" => processes.sort_by_key(|b| std::cmp::Reverse(b.1.memory())),
            _ => processes.sort_by(|a, b| a.1.name().cmp(b.1.name())),
        }

        let mut output = Vec::new();
        output.push(format!(
            "{:<8} {:<30} {:<10} {:<12} {}",
            "PID", "Name", "CPU%", "Memory (MB)", "Status"
        ));
        output.push("-".repeat(75));

        for (pid, proc) in processes.iter().take(limit) {
            let status = match proc.status() {
                sysinfo::ProcessStatus::Run => "Running",
                sysinfo::ProcessStatus::Sleep => "Sleep",
                sysinfo::ProcessStatus::Idle => "Idle",
                sysinfo::ProcessStatus::Zombie => "Zombie",
                sysinfo::ProcessStatus::Stop => "Stopped",
                _ => "Unknown",
            };
            output.push(format!(
                "{:<8} {:<30} {:<10.1} {:<12.1} {}",
                pid.as_u32(),
                truncate_str(&proc.name().to_string_lossy(), 30),
                proc.cpu_usage(),
                proc.memory() as f64 / 1_000_000.0,
                status,
            ));
        }

        Ok(ToolResult::success("list_processes", output.join("\n")))
    }
}

/// Get detailed info about a specific process.
pub struct ProcessInfo;

#[async_trait]
impl Tool for ProcessInfo {
    fn name(&self) -> &str {
        "process_info"
    }

    fn description(&self) -> &str {
        "Get detailed information about a specific process by PID."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pid": {
                    "type": "integer",
                    "description": "Process ID"
                }
            },
            "required": ["pid"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::ReadOnly
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let pid_val = input["pid"]
            .as_u64()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'pid' parameter".to_string(),
            })?;
        let pid = sysinfo::Pid::from(pid_val as usize);

        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let proc = sys.process(pid).ok_or_else(|| MarvisError::ToolError {
            tool: self.name().to_string(),
            message: format!("Process with PID {} not found", pid_val),
        })?;

        let mut info = Vec::new();
        info.push(format!("PID: {}", pid_val));
        info.push(format!("Name: {}", proc.name().to_string_lossy()));
        info.push(format!("CPU Usage: {:.2}%", proc.cpu_usage()));
        info.push(format!("Memory: {} bytes", proc.memory()));
        info.push(format!("Virtual Memory: {} bytes", proc.virtual_memory()));
        info.push(format!(
            "Disk Usage: read={} bytes, written={} bytes",
            proc.disk_usage().read_bytes,
            proc.disk_usage().written_bytes
        ));
        info.push(format!("Run Time: {} seconds", proc.run_time()));
        if let Some(exe) = proc.exe() {
            info.push(format!("Executable: {}", exe.display()));
        }
        if let Some(cwd) = proc.cwd() {
            info.push(format!("Working Dir: {}", cwd.display()));
        }

        Ok(ToolResult::success("process_info", info.join("\n")))
    }
}

/// Get CPU information.
pub struct CpuInfo;

#[async_trait]
impl Tool for CpuInfo {
    fn name(&self) -> &str {
        "cpu_info"
    }

    fn description(&self) -> &str {
        "Get CPU information including name, core count, and current usage."
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
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let mut info = Vec::new();
        let cpus = sys.cpus();

        info.push(format!("CPU Count: {} logical cores", cpus.len()));
        if let Some(cpu) = cpus.first() {
            info.push(format!("CPU Brand: {}", cpu.brand()));
            info.push(format!("CPU Frequency: {} MHz", cpu.frequency()));
        }

        info.push(String::new());
        info.push("Per-core usage:".to_string());
        for (i, cpu) in cpus.iter().enumerate() {
            info.push(format!("  Core {:>2}: {:.1}%", i, cpu.cpu_usage()));
        }

        // Memory info too since it's often requested alongside CPU
        info.push(String::new());
        info.push(format!(
            "Total Memory: {:.2} GB",
            sys.total_memory() as f64 / 1_000_000_000.0
        ));
        info.push(format!(
            "Used Memory: {:.2} GB",
            sys.used_memory() as f64 / 1_000_000_000.0
        ));
        info.push(format!(
            "Available Memory: {:.2} GB",
            sys.available_memory() as f64 / 1_000_000_000.0
        ));

        Ok(ToolResult::success("cpu_info", info.join("\n")))
    }
}

/// Get memory information.
pub struct MemoryInfo;

#[async_trait]
impl Tool for MemoryInfo {
    fn name(&self) -> &str {
        "memory_info"
    }

    fn description(&self) -> &str {
        "Get system memory usage information."
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
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let total = sys.total_memory();
        let used = sys.used_memory();
        let available = sys.available_memory();
        let free = total - used;

        let mut info = Vec::new();
        info.push(format!("Total Memory:     {:>10.2} GB", total as f64 / 1e9));
        info.push(format!("Used Memory:      {:>10.2} GB", used as f64 / 1e9));
        info.push(format!("Free Memory:      {:>10.2} GB", free as f64 / 1e9));
        info.push(format!(
            "Available Memory: {:>10.2} GB",
            available as f64 / 1e9
        ));
        info.push(format!(
            "Usage:            {:>10.1}%",
            (used as f64 / total as f64) * 100.0
        ));
        info.push(format!(
            "Total Swap:       {:>10.2} GB",
            sys.total_swap() as f64 / 1e9
        ));
        info.push(format!(
            "Used Swap:        {:>10.2} GB",
            sys.used_swap() as f64 / 1e9
        ));

        Ok(ToolResult::success("memory_info", info.join("\n")))
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_processes_default() {
        let tool = ListProcesses;
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("PID"));
        assert!(result.content.contains("Name"));
    }

    #[tokio::test]
    async fn test_list_processes_with_limit() {
        let tool = ListProcesses;
        let result = tool
            .execute(serde_json::json!({"limit": 5, "sort_by": "cpu"}))
            .await
            .unwrap();
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_cpu_info() {
        let tool = CpuInfo;
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("CPU"));
    }

    #[tokio::test]
    async fn test_memory_info() {
        let tool = MemoryInfo;
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("Memory"));
    }

    #[tokio::test]
    async fn test_process_info_nonexistent() {
        let tool = ProcessInfo;
        let result = tool.execute(serde_json::json!({"pid": 9999999})).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_risk_levels() {
        assert_eq!(ListProcesses.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(ProcessInfo.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(CpuInfo.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(MemoryInfo.risk_level(), RiskLevel::ReadOnly);
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8), "hello...");
    }
}
