//! System information tools.

use async_trait::async_trait;
use marvis_core::{MarvisError, RiskLevel, Tool, ToolResult};

/// Get overall system summary.
pub struct SystemInfo;

#[async_trait]
impl Tool for SystemInfo {
    fn name(&self) -> &str {
        "system_info"
    }

    fn description(&self) -> &str {
        "Get an overview of the system: OS, hostname, uptime, CPU, memory, and disk."
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
        let mut info = Vec::new();

        // OS info
        info.push(format!("OS: {}", std::env::consts::OS));
        info.push(format!("Arch: {}", std::env::consts::ARCH));

        // System info via sysinfo
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        info.push(format!(
            "System name: {}",
            sysinfo::System::name().unwrap_or_else(|| "Unknown".to_string())
        ));
        info.push(format!(
            "Kernel version: {}",
            sysinfo::System::kernel_version().unwrap_or_else(|| "Unknown".to_string())
        ));
        info.push(format!(
            "OS version: {}",
            sysinfo::System::os_version().unwrap_or_else(|| "Unknown".to_string())
        ));

        // Uptime
        let uptime = sysinfo::System::uptime();
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        info.push(format!("Uptime: {}h {}m", hours, minutes));

        // CPU
        info.push(format!("CPU cores: {}", sys.cpus().len()));

        // Memory
        let total_mem = sys.total_memory() as f64 / 1_000_000_000.0;
        let used_mem = sys.used_memory() as f64 / 1_000_000_000.0;
        info.push(format!("Memory: {:.1}/{:.1} GB", used_mem, total_mem));

        // Current directory
        if let Ok(cwd) = std::env::current_dir() {
            info.push(format!("Working directory: {}", cwd.display()));
        }

        Ok(ToolResult::success("system_info", info.join("\n")))
    }
}

/// Read an environment variable.
pub struct EnvVariable;

#[async_trait]
impl Tool for EnvVariable {
    fn name(&self) -> &str {
        "env_variable"
    }

    fn description(&self) -> &str {
        "Read the value of an environment variable."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name of the environment variable"
                }
            },
            "required": ["name"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::ReadOnly
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let name = input["name"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'name' parameter".to_string(),
            })?;

        match std::env::var(name) {
            Ok(value) => Ok(ToolResult::success("env_variable", value)),
            Err(std::env::VarError::NotPresent) => Ok(ToolResult::success(
                "env_variable",
                format!("Environment variable '{}' is not set.", name),
            )),
            Err(e) => Ok(ToolResult::error(
                "env_variable",
                format!("Cannot read '{}': {}", name, e),
            )),
        }
    }
}

/// Execute a shell command.
pub struct RunCommand;

#[async_trait]
impl Tool for RunCommand {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command and return the output. Use with caution."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Command arguments"
                }
            },
            "required": ["command"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Normal
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let command = input["command"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'command' parameter".to_string(),
            })?;

        let args: Vec<String> = input["args"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let output = std::process::Command::new(command)
            .args(&args)
            .output()
            .map_err(|e| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: format!("Failed to execute '{}': {}", command, e),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&format!("STDOUT:\n{}", stdout));
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(&format!("STDERR:\n{}", stderr));
        }
        if result.is_empty() {
            result.push_str(&format!(
                "Command '{}' executed with no output (exit code: {})",
                command,
                output.status.code().unwrap_or(-1)
            ));
        }

        if output.status.success() {
            Ok(ToolResult::success("run_command", result))
        } else {
            Ok(ToolResult::error("run_command", result))
        }
    }
}

/// Execute a shell command with platform-aware handling and browser interception.
///
/// On Windows uses `cmd /C`, on Unix uses `sh -c`.
/// Automatically redirects browser commands (msedge, chrome, firefox, etc.) to open_browser.
pub struct RunShell;

#[async_trait]
impl Tool for RunShell {
    fn name(&self) -> &str {
        "run_shell"
    }

    fn description(&self) -> &str {
        "Execute a shell command. On Windows uses cmd /C. \
         Use ONLY for: echo, dir, type, mkdir, rmdir, copy, move, del. \
         NEVER use for browsers or URLs — use open_browser instead."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                }
            },
            "required": ["command"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Normal
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let cmd = input["command"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'command' parameter".to_string(),
            })?;

        let lower = cmd.to_lowercase();

        // Intercept browser commands → redirect to open_browser
        if lower.contains("msedge")
            || lower.contains("chrome")
            || lower.contains("firefox")
            || lower.contains("iexplore")
            || lower.contains("start http")
            || lower.contains("explorer http")
            || lower.contains("edge")
            || lower.contains("browser")
        {
            let url = if let Some(pos) = lower.find("http") {
                cmd[pos..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("https://www.google.com")
            } else if lower.contains("google") {
                "https://www.google.com"
            } else {
                let query = cmd.split_whitespace().last().unwrap_or("search");
                return Ok(ToolResult::error(
                    "run_shell",
                    format!(
                        "Browser commands should use open_browser. Try: open_browser with search_query=\"{}\"",
                        query
                    ),
                ));
            };

            // Delegate to open_browser via the shared helper
            match crate::web::open_with_shell(url) {
                Ok(()) => {
                    return Ok(ToolResult::success(
                        "run_shell",
                        format!("Browser opened: {}", url),
                    ));
                }
                Err(e) => {
                    return Ok(ToolResult::error(
                        "run_shell",
                        format!(
                            "Failed to open browser: {}. Please use open_browser tool instead.",
                            e
                        ),
                    ));
                }
            }
        }

        // Execute the command
        let output = if cfg!(target_os = "windows") {
            std::process::Command::new("cmd").args(["/C", cmd]).output()
        } else {
            std::process::Command::new("sh").args(["-c", cmd]).output()
        };

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let mut r = String::new();
                if !stdout.is_empty() {
                    r.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    r.push_str(&format!("\nSTDERR: {}", stderr));
                }
                if r.is_empty() {
                    r = format!("Exit code: {}", o.status.code().unwrap_or(-1));
                }
                if o.status.success() {
                    Ok(ToolResult::success("run_shell", r))
                } else {
                    Ok(ToolResult::error("run_shell", r))
                }
            }
            Err(e) => Ok(ToolResult::error("run_shell", format!("Failed: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_info() {
        let tool = SystemInfo;
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("OS"));
    }

    #[tokio::test]
    async fn test_env_variable_valid() {
        let tool = EnvVariable;
        let result = tool
            .execute(serde_json::json!({"name": "PATH"}))
            .await
            .unwrap();
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_env_variable_not_set() {
        let tool = EnvVariable;
        let result = tool
            .execute(serde_json::json!({"name": "THIS_VAR_DOES_NOT_EXIST_XYZ"}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("not set"));
    }

    #[tokio::test]
    async fn test_env_variable_missing_name() {
        let tool = EnvVariable;
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_command_echo() {
        let tool = RunCommand;
        let (cmd, args): (&str, Vec<&str>) = if cfg!(target_os = "windows") {
            ("cmd", vec!["/C", "echo", "hello", "world"])
        } else {
            ("echo", vec!["hello", "world"])
        };
        let result = tool
            .execute(serde_json::json!({
                "command": cmd,
                "args": args
            }))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_run_command_nonexistent() {
        let tool = RunCommand;
        let result = tool
            .execute(serde_json::json!({"command": "nonexistent_command_xyz"}))
            .await;
        // May return Err or Ok(ToolResult { is_error: true })
        if let Ok(r) = result {
            assert!(r.is_error);
        }
    }

    #[tokio::test]
    async fn test_run_shell_echo() {
        let tool = RunShell;
        let result = tool
            .execute(serde_json::json!({"command": "echo hello"}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_run_shell_browser_interception() {
        let tool = RunShell;
        let result = tool
            .execute(serde_json::json!({"command": "msedge https://example.com"}))
            .await
            .unwrap();
        // Browser commands should be intercepted (redirect or error, never crash)
        // Result may vary: success=browser opened, error=use open_browser instead
        if result.is_error {
            assert!(result.content.contains("open_browser"));
        }
    }

    #[test]
    fn test_tool_risk_levels() {
        assert_eq!(SystemInfo.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(EnvVariable.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(RunCommand.risk_level(), RiskLevel::Normal);
        assert_eq!(RunShell.risk_level(), RiskLevel::Normal);
    }
}
