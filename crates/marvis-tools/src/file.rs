//! File operation tools.

use async_trait::async_trait;
use marvis_core::{MarvisError, RiskLevel, Tool, ToolResult};
use std::path::PathBuf;

/// Read the contents of a file.
pub struct ReadFile;

#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file at the given path. Supports optional offset and limit for large files."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (0-based)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read"
                }
            },
            "required": ["path"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::ReadOnly
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'path' parameter".to_string(),
            })?;

        let content = std::fs::read_to_string(path).map_err(|e| MarvisError::ToolError {
            tool: self.name().to_string(),
            message: format!("Cannot read file '{}': {}", path, e),
        })?;

        let offset = input["offset"].as_u64().unwrap_or(0) as usize;
        let limit = input["limit"].as_u64().map(|l| l as usize);

        let lines: Vec<&str> = content.lines().skip(offset).collect();
        let lines = if let Some(limit) = limit {
            lines.into_iter().take(limit).collect::<Vec<_>>()
        } else {
            lines
        };

        Ok(ToolResult::success("read_file", lines.join("\n")))
    }
}

/// Write content to a file.
pub struct WriteFile;

#[async_trait]
impl Tool for WriteFile {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file. Creates the file if it does not exist, overwrites it if it does."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Normal
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'path' parameter".to_string(),
            })?;
        let content = input["content"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'content' parameter".to_string(),
            })?;

        std::fs::write(path, content).map_err(|e| MarvisError::ToolError {
            tool: self.name().to_string(),
            message: format!("Cannot write file '{}': {}", path, e),
        })?;

        Ok(ToolResult::success(
            "write_file",
            format!("Successfully wrote to '{}'", path),
        ))
    }
}

/// List contents of a directory.
pub struct ListDirectory;

#[async_trait]
impl Tool for ListDirectory {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List the contents of a directory. Optionally list recursively."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the directory to list"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Whether to list recursively"
                }
            },
            "required": ["path"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::ReadOnly
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'path' parameter".to_string(),
            })?;
        let recursive = input["recursive"].as_bool().unwrap_or(false);

        let entries = if recursive {
            list_recursive(path)?
        } else {
            list_dir(path)?
        };

        Ok(ToolResult::success("list_directory", entries.join("\n")))
    }
}

fn list_dir(path: &str) -> Result<Vec<String>, MarvisError> {
    let mut entries = Vec::new();
    let dir = std::fs::read_dir(path).map_err(|e| MarvisError::ToolError {
        tool: "list_directory".to_string(),
        message: format!("Cannot read directory '{}': {}", path, e),
    })?;

    for entry in dir {
        let entry = entry.map_err(MarvisError::IoError)?;
        let file_type = entry.file_type().map_err(MarvisError::IoError)?;
        let type_char = if file_type.is_dir() { "d" } else { "f" };
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata().ok();
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        entries.push(format!("{} {:>10} {}", type_char, size, name));
    }

    entries.sort();
    Ok(entries)
}

fn list_recursive(path: &str) -> Result<Vec<String>, MarvisError> {
    let mut entries = Vec::new();
    entries.push(format!("📁 {}/", path));

    fn walk(path: &PathBuf, entries: &mut Vec<String>, prefix: &str) -> Result<(), std::io::Error> {
        let dir = std::fs::read_dir(path)?;
        let mut items: Vec<_> = dir.collect::<Result<Vec<_>, _>>()?;
        items.sort_by_key(|e| e.file_name());

        for (i, entry) in items.iter().enumerate() {
            let is_last = i == items.len() - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let child_prefix = if is_last { "    " } else { "│   " };

            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                entries.push(format!("{}{}📁 {}/", prefix, connector, name));
                walk(
                    &entry.path(),
                    entries,
                    &format!("{}{}", prefix, child_prefix),
                )?;
            } else {
                let metadata = entry.metadata()?;
                entries.push(format!("{}📄 {} ({} bytes)", prefix, name, metadata.len()));
            }
        }
        Ok(())
    }

    walk(&PathBuf::from(path), &mut entries, "").map_err(MarvisError::IoError)?;
    Ok(entries)
}

/// Delete a file or empty directory.
pub struct DeleteFile;

#[async_trait]
impl Tool for DeleteFile {
    fn name(&self) -> &str {
        "delete_file"
    }

    fn description(&self) -> &str {
        "Delete a file at the given path. This operation cannot be undone."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to delete"
                }
            },
            "required": ["path"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Dangerous
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'path' parameter".to_string(),
            })?;

        let metadata = std::fs::metadata(path).map_err(|e| MarvisError::ToolError {
            tool: self.name().to_string(),
            message: format!("Cannot access '{}': {}", path, e),
        })?;

        if metadata.is_dir() {
            std::fs::remove_dir(path).map_err(|e| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: format!("Cannot remove directory '{}': {}", path, e),
            })?;
        } else {
            std::fs::remove_file(path).map_err(|e| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: format!("Cannot delete file '{}': {}", path, e),
            })?;
        }

        Ok(ToolResult::success(
            "delete_file",
            format!("Successfully deleted '{}'", path),
        ))
    }
}

/// Get file metadata.
pub struct FileInfo;

#[async_trait]
impl Tool for FileInfo {
    fn name(&self) -> &str {
        "file_info"
    }

    fn description(&self) -> &str {
        "Get metadata about a file (size, modification time, permissions)"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file"
                }
            },
            "required": ["path"]
        })
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::ReadOnly
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, MarvisError> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| MarvisError::ToolError {
                tool: self.name().to_string(),
                message: "Missing 'path' parameter".to_string(),
            })?;

        let metadata = std::fs::metadata(path).map_err(|e| MarvisError::ToolError {
            tool: self.name().to_string(),
            message: format!("Cannot access '{}': {}", path, e),
        })?;

        let mut info = Vec::new();
        info.push(format!("Path: {}", path));
        info.push(format!("Size: {} bytes", metadata.len()));
        info.push(format!("Is directory: {}", metadata.is_dir()));
        info.push(format!("Is file: {}", metadata.is_file()));
        info.push(format!("Read only: {}", metadata.permissions().readonly()));

        if let Ok(modified) = metadata.modified() {
            info.push(format!(
                "Modified: {:?}",
                modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| format!("{} seconds since epoch", d.as_secs()))
                    .unwrap_or_else(|_| "unknown".to_string())
            ));
        }

        Ok(ToolResult::success("file_info", info.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn temp_file(content: &str) -> String {
        let path = std::env::temp_dir().join(format!("marvis_test_{}.txt", uuid::Uuid::new_v4()));
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn test_read_file_exists() {
        let path = temp_file("hello world\nline two\nline three");
        let tool = ReadFile;
        let result = tool
            .execute(serde_json::json!({"path": path}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("hello world"));
        let _ = fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_read_file_not_found() {
        let tool = ReadFile;
        let result = tool
            .execute(serde_json::json!({"path": "/nonexistent/file.txt"}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_file_with_offset_and_limit() {
        let path = temp_file("line0\nline1\nline2\nline3\nline4");
        let tool = ReadFile;
        let result = tool
            .execute(serde_json::json!({"path": path, "offset": 1, "limit": 2}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(!result.content.contains("line0"));
        assert!(result.content.contains("line1"));
        assert!(result.content.contains("line2"));
        let _ = fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_write_and_read_file() {
        let path =
            std::env::temp_dir().join(format!("marvis_write_test_{}.txt", uuid::Uuid::new_v4()));
        let path_str = path.to_string_lossy().to_string();

        let write_tool = WriteFile;
        let result = write_tool
            .execute(serde_json::json!({"path": path_str, "content": "test content"}))
            .await
            .unwrap();
        assert!(!result.is_error);

        let read_tool = ReadFile;
        let result = read_tool
            .execute(serde_json::json!({"path": path_str}))
            .await
            .unwrap();
        assert!(result.content.contains("test content"));

        let _ = fs::remove_file(path_str);
    }

    #[tokio::test]
    async fn test_list_directory() {
        let tool = ListDirectory;
        let result = tool
            .execute(serde_json::json!({"path": ".", "recursive": false}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_file_info() {
        let path = temp_file("sample content");
        let tool = FileInfo;
        let result = tool
            .execute(serde_json::json!({"path": path}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("bytes"));
        let _ = fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_delete_file() {
        let path = temp_file("to be deleted");
        let tool = DeleteFile;
        let result = tool
            .execute(serde_json::json!({"path": path}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(!fs::metadata(path).is_ok()); // Should no longer exist
    }

    #[test]
    fn test_tool_risk_levels() {
        assert_eq!(ReadFile.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(WriteFile.risk_level(), RiskLevel::Normal);
        assert_eq!(DeleteFile.risk_level(), RiskLevel::Dangerous);
        assert_eq!(ListDirectory.risk_level(), RiskLevel::ReadOnly);
        assert_eq!(FileInfo.risk_level(), RiskLevel::ReadOnly);
    }

    #[test]
    fn test_tool_names() {
        assert_eq!(ReadFile.name(), "read_file");
        assert_eq!(WriteFile.name(), "write_file");
        assert_eq!(ListDirectory.name(), "list_directory");
        assert_eq!(DeleteFile.name(), "delete_file");
        assert_eq!(FileInfo.name(), "file_info");
    }
}
