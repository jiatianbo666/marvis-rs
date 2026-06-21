//! Sandbox isolation for tool execution.

use std::path::{Path, PathBuf};

/// A simple sandbox that restricts file operations to a specific directory.
pub struct Sandbox {
    /// Allowed working directory. Operations outside this are denied.
    workspace: PathBuf,
    /// Whether the sandbox is enabled.
    enabled: bool,
}

impl Sandbox {
    /// Create a new sandbox restricted to the given workspace.
    pub fn new(workspace: impl Into<PathBuf>) -> Self {
        Self {
            workspace: workspace.into(),
            enabled: true,
        }
    }

    /// Create a disabled sandbox (no restrictions).
    pub fn disabled() -> Self {
        Self {
            workspace: PathBuf::from("/"),
            enabled: false,
        }
    }

    /// Check if a path is within the allowed workspace.
    pub fn is_allowed(&self, path: &Path) -> bool {
        if !self.enabled {
            return true;
        }

        // Canonicalize both paths for comparison
        let canonical_workspace = self
            .workspace
            .canonicalize()
            .unwrap_or_else(|_| self.workspace.clone());
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        canonical_path.starts_with(&canonical_workspace)
    }

    /// Resolve a path relative to the workspace.
    pub fn resolve(&self, path: impl AsRef<Path>) -> PathBuf {
        let p = path.as_ref();
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.workspace.join(p)
        }
    }

    /// Enable or disable the sandbox.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if the sandbox is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the workspace directory.
    pub fn workspace(&self) -> &Path {
        &self.workspace
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self {
            workspace: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_disabled_sandbox_allows_all() {
        let sandbox = Sandbox::disabled();
        assert!(!sandbox.is_enabled());
        assert!(sandbox.is_allowed(Path::new("/etc/passwd")));
        assert!(sandbox.is_allowed(Path::new("C:\\Windows")));
    }

    #[test]
    fn test_enabled_sandbox_restricts() {
        let sandbox = Sandbox::new(".");
        assert!(sandbox.is_enabled());
        // Current directory should be allowed
        assert!(sandbox.is_allowed(Path::new(".")));
    }

    #[test]
    fn test_resolve_relative_path() {
        let sandbox = Sandbox::new("/workspace");
        let resolved = sandbox.resolve("subdir/file.txt");
        assert_eq!(resolved, PathBuf::from("/workspace/subdir/file.txt"));
    }

    #[test]
    fn test_resolve_absolute_path() {
        let sandbox = Sandbox::new("/workspace");
        let resolved = sandbox.resolve("/absolute/path.txt");
        assert_eq!(resolved, PathBuf::from("/absolute/path.txt"));
    }

    #[test]
    fn test_workspace_path() {
        let sandbox = Sandbox::new("/my/app");
        assert_eq!(sandbox.workspace(), Path::new("/my/app"));
    }

    #[test]
    fn test_set_enabled() {
        let mut sandbox = Sandbox::new(".");
        sandbox.set_enabled(false);
        assert!(!sandbox.is_enabled());
        sandbox.set_enabled(true);
        assert!(sandbox.is_enabled());
    }
}
