use thiserror::Error;

/// Top-level error type for the Marvis system.
#[derive(Debug, Error)]
pub enum MarvisError {
    /// AI API call failed.
    #[error("AI error: {0}")]
    AiError(String),

    /// Tool execution failed.
    #[error("Tool '{tool}' error: {message}")]
    ToolError { tool: String, message: String },

    /// Configuration error.
    #[error("Config error: {0}")]
    ConfigError(String),

    /// Permission denied for a tool.
    #[error("Permission denied for '{tool}': {reason}")]
    PermissionDenied { tool: String, reason: String },

    /// Session management error.
    #[error("Session error: {0}")]
    SessionError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Agent loop exceeded max iterations.
    #[error("Max agent loop iterations ({0}) exceeded")]
    MaxIterationsExceeded(usize),

    /// User cancelled the operation.
    #[error("Operation cancelled by user")]
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_error_display() {
        let err = MarvisError::AiError("connection refused".to_string());
        assert!(err.to_string().contains("AI error"));
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn test_tool_error_display() {
        let err = MarvisError::ToolError {
            tool: "read_file".to_string(),
            message: "file not found".to_string(),
        };
        assert!(err.to_string().contains("read_file"));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_permission_denied_display() {
        let err = MarvisError::PermissionDenied {
            tool: "delete_file".to_string(),
            reason: "path is protected".to_string(),
        };
        assert!(err.to_string().contains("Permission denied"));
        assert!(err.to_string().contains("delete_file"));
    }

    #[test]
    fn test_config_error_display() {
        let err = MarvisError::ConfigError("missing API key".to_string());
        assert!(err.to_string().contains("Config error"));
        assert!(err.to_string().contains("missing API key"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let err: MarvisError = io_err.into();
        assert!(matches!(err, MarvisError::IoError(_)));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err: MarvisError = json_err.into();
        assert!(matches!(err, MarvisError::JsonError(_)));
    }

    #[test]
    fn test_max_iterations_exceeded_display() {
        let err = MarvisError::MaxIterationsExceeded(10);
        assert!(err.to_string().contains("10"));
    }

    #[test]
    fn test_cancelled_display() {
        let err = MarvisError::Cancelled;
        assert!(err.to_string().contains("cancelled"));
    }

    #[test]
    fn test_session_error_display() {
        let err = MarvisError::SessionError("save failed".to_string());
        assert!(err.to_string().contains("Session error"));
        assert!(err.to_string().contains("save failed"));
    }
}
