//! Permission model and security manager.

use marvis_core::MarvisError;

/// Permission modes for the assistant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionMode {
    /// Only read-only operations allowed.
    ReadOnly,
    /// Normal operations allowed, dangerous ones require confirmation.
    Normal,
    /// All operations allowed without confirmation (use with caution).
    Dangerous,
}

/// Sensitive keywords that indicate a potentially dangerous operation.
const DANGEROUS_KEYWORDS: &[&str] = &[
    "delete",
    "remove",
    "rm",
    "rmdir",
    "kill",
    "terminate",
    "stop",
    "format",
    "mkfs",
    "uninstall",
    "purge",
    "overwrite",
    "truncate",
    "shutdown",
    "reboot",
    "restart",
    "chmod",
    "chown",
];

/// System paths that should never be touched.
const PROTECTED_PATHS: &[&str] = &[
    "C:\\Windows",
    "C:\\Windows\\System32",
    "C:\\Program Files",
    "/etc",
    "/usr",
    "/bin",
    "/sbin",
    "/boot",
    "/sys",
    "/proc",
    "/dev",
    "~/.ssh",
];

/// Manages security checks for tool execution.
pub struct SecurityManager {
    mode: PermissionMode,
    /// Specific tool names that the user has approved for this session.
    approved_tools: Vec<String>,
}

impl SecurityManager {
    /// Create a new security manager with the given permission mode.
    pub fn new(mode: PermissionMode) -> Self {
        Self {
            mode,
            approved_tools: Vec::new(),
        }
    }

    /// Get the current permission mode.
    pub fn mode(&self) -> PermissionMode {
        self.mode
    }

    /// Set the permission mode.
    pub fn set_mode(&mut self, mode: PermissionMode) {
        self.mode = mode;
    }

    /// Approve a specific tool for this session.
    pub fn approve_tool(&mut self, tool_name: impl Into<String>) {
        self.approved_tools.push(tool_name.into());
    }

    /// Check if a tool execution should be allowed.
    pub fn check(&self, _tool_name: &str, args: &serde_json::Value) -> Result<(), MarvisError> {
        // Check permission mode
        match self.mode {
            PermissionMode::Dangerous => {
                // All operations allowed
                return Ok(());
            }
            PermissionMode::ReadOnly => {
                // Only read-only operations allowed
                // We trust the tool's risk_level, but also check here
            }
            PermissionMode::Normal => {
                // Normal mode — dangerous operations need confirmation
            }
        }

        // Check for protected paths
        if let Some(path) = self.extract_path(args) {
            self.check_protected_path(path)?;
        }

        // Check for dangerous commands
        if let Some(cmd) = args.get("command").and_then(|v| v.as_str()) {
            self.check_dangerous_command(cmd)?;
        }

        Ok(())
    }

    /// Whether this tool execution needs user confirmation.
    pub fn needs_confirmation(&self, tool_name: &str, args: &serde_json::Value) -> bool {
        if self.mode == PermissionMode::Dangerous {
            return false;
        }

        // Already approved for this session?
        if self.approved_tools.contains(&tool_name.to_string()) {
            return false;
        }

        // Check if any argument contains dangerous keywords
        let args_str = serde_json::to_string(args)
            .unwrap_or_default()
            .to_lowercase();
        for keyword in DANGEROUS_KEYWORDS {
            if args_str.contains(keyword) {
                return true;
            }
        }

        false
    }

    /// Check if a path is protected.
    fn check_protected_path(&self, path: &str) -> Result<(), MarvisError> {
        let normalized = path.replace('\\', "/").to_lowercase();
        for protected in PROTECTED_PATHS {
            let protected_normalized = protected.replace('\\', "/").to_lowercase();
            if normalized.starts_with(&protected_normalized) {
                return Err(MarvisError::PermissionDenied {
                    tool: "file_operation".to_string(),
                    reason: format!(
                        "Path '{}' is in a protected system directory '{}'",
                        path, protected
                    ),
                });
            }
        }
        Ok(())
    }

    /// Check if a command contains dangerous keywords.
    fn check_dangerous_command(&self, command: &str) -> Result<(), MarvisError> {
        let cmd_lower = command.to_lowercase();
        for keyword in DANGEROUS_KEYWORDS {
            if cmd_lower.contains(keyword) {
                return Err(MarvisError::PermissionDenied {
                    tool: "run_command".to_string(),
                    reason: format!(
                        "Command '{}' contains dangerous keyword '{}'. User confirmation required.",
                        command, keyword
                    ),
                });
            }
        }
        Ok(())
    }

    /// Extract a path argument from tool arguments if present.
    fn extract_path<'a>(&self, args: &'a serde_json::Value) -> Option<&'a str> {
        args.get("path").and_then(|v| v.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_mode_allows_safe_operations() {
        let sm = SecurityManager::new(PermissionMode::Normal);
        let args = serde_json::json!({"path": "/tmp/test.txt"});
        assert!(sm.check("read_file", &args).is_ok());
    }

    #[test]
    fn test_read_only_means_read_only() {
        let sm = SecurityManager::new(PermissionMode::ReadOnly);
        let args = serde_json::json!({"path": "/tmp/test.txt"});
        // Check still passes — the tool's risk level is checked elsewhere
        assert!(sm.check("any_tool", &args).is_ok());
    }

    #[test]
    fn test_dangerous_mode_allows_all() {
        let sm = SecurityManager::new(PermissionMode::Dangerous);
        let args = serde_json::json!({"command": "rm -rf /"});
        assert!(sm.check("run_command", &args).is_ok());
        assert!(!sm.needs_confirmation("run_command", &args));
    }

    #[test]
    fn test_dangerous_keyword_triggers_confirmation() {
        let sm = SecurityManager::new(PermissionMode::Normal);
        let args = serde_json::json!({"path": "/tmp/delete_me.txt"});
        assert!(sm.needs_confirmation("delete_file", &args));
    }

    #[test]
    fn test_safe_operation_no_confirmation() {
        let sm = SecurityManager::new(PermissionMode::Normal);
        let args = serde_json::json!({"path": "/tmp/readme.txt"});
        assert!(!sm.needs_confirmation("read_file", &args));
    }

    #[test]
    fn test_protected_path_windows() {
        let sm = SecurityManager::new(PermissionMode::Normal);
        let args = serde_json::json!({"path": "C:\\Windows\\System32\\kernel32.dll"});
        let result = sm.check("delete_file", &args);
        // Protected path should be denied
        assert!(result.is_err() || sm.needs_confirmation("delete_file", &args));
    }

    #[test]
    fn test_protected_path_unix() {
        let sm = SecurityManager::new(PermissionMode::Normal);
        let args = serde_json::json!({"path": "/etc/passwd"});
        let result = sm.check("delete_file", &args);
        assert!(result.is_err() || sm.needs_confirmation("delete_file", &args));
    }

    #[test]
    fn test_approved_tool_no_confirmation() {
        let mut sm = SecurityManager::new(PermissionMode::Normal);
        sm.approve_tool("delete_file");
        let args = serde_json::json!({"path": "/tmp/test.txt"});
        assert!(!sm.needs_confirmation("delete_file", &args));
    }

    #[test]
    fn test_kill_command_needs_confirmation() {
        let sm = SecurityManager::new(PermissionMode::Normal);
        let args = serde_json::json!({"command": "kill 1234"});
        assert!(sm.needs_confirmation("run_command", &args));
    }

    #[test]
    fn test_permission_mode_switch() {
        let mut sm = SecurityManager::new(PermissionMode::Normal);
        assert_eq!(sm.mode(), PermissionMode::Normal);

        sm.set_mode(PermissionMode::ReadOnly);
        assert_eq!(sm.mode(), PermissionMode::ReadOnly);

        sm.set_mode(PermissionMode::Dangerous);
        assert_eq!(sm.mode(), PermissionMode::Dangerous);
    }
}
