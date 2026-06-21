//! Persistent session storage.

use crate::history::ConversationHistory;
use marvis_core::MarvisError;
use std::path::PathBuf;

/// Persists and loads conversation sessions to/from disk.
pub struct SessionStorage {
    /// Directory where sessions are stored.
    storage_dir: PathBuf,
}

impl SessionStorage {
    /// Create a new session storage with the given directory.
    pub fn new(storage_dir: impl Into<PathBuf>) -> Self {
        let dir = storage_dir.into();
        // Ensure directory exists
        if let Err(e) = std::fs::create_dir_all(&dir) {
            // Ignore if already exists
            log::warn!("Could not create session storage dir {:?}: {}", dir, e);
        }
        Self { storage_dir: dir }
    }

    /// Save a session to disk.
    pub fn save(&self, id: &str, history: &ConversationHistory) -> Result<(), MarvisError> {
        let json = history
            .to_json()
            .map_err(|e| MarvisError::SessionError(format!("Serialization failed: {e}")))?;

        let path = self.session_path(id);
        std::fs::write(&path, json).map_err(|e| {
            MarvisError::SessionError(format!("Failed to write session file {:?}: {}", path, e))
        })?;

        log::info!("Session '{}' saved to {:?}", id, path);
        Ok(())
    }

    /// Load a session from disk.
    pub fn load(&self, id: &str) -> Result<ConversationHistory, MarvisError> {
        let path = self.session_path(id);
        let json = std::fs::read_to_string(&path).map_err(|e| {
            MarvisError::SessionError(format!("Failed to read session file {:?}: {}", path, e))
        })?;

        ConversationHistory::from_json(&json)
            .map_err(|e| MarvisError::SessionError(format!("Deserialization failed: {e}")))
    }

    /// List all available session IDs.
    pub fn list_sessions(&self) -> Result<Vec<String>, MarvisError> {
        let mut sessions = Vec::new();
        let entries = std::fs::read_dir(&self.storage_dir).map_err(|e| {
            MarvisError::SessionError(format!(
                "Cannot read storage directory {:?}: {}",
                self.storage_dir, e
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(MarvisError::IoError)?;
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                if let Some(name) = entry.path().file_stem() {
                    sessions.push(name.to_string_lossy().to_string());
                }
            }
        }

        sessions.sort();
        Ok(sessions)
    }

    /// Delete a session file.
    pub fn delete(&self, id: &str) -> Result<(), MarvisError> {
        let path = self.session_path(id);
        std::fs::remove_file(&path).map_err(|e| {
            MarvisError::SessionError(format!("Failed to delete session file {:?}: {}", path, e))
        })?;
        Ok(())
    }

    fn session_path(&self, id: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.json", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::ConversationHistory;
    use std::env;

    fn temp_dir() -> PathBuf {
        let dir = env::temp_dir().join(format!("marvis_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    #[test]
    fn test_save_and_load_session() {
        let dir = temp_dir();
        let storage = SessionStorage::new(&dir);

        let mut history = ConversationHistory::new();
        history.add_user("hello world");
        history.add_assistant("hi there", None);

        storage.save("test_session", &history).unwrap();

        let loaded = storage.load("test_session").unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.messages()[0].content, "hello world");

        // Cleanup
        storage.delete("test_session").unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_sessions() {
        let dir = temp_dir();
        let storage = SessionStorage::new(&dir);

        let mut history = ConversationHistory::new();
        history.add_user("test");

        storage.save("session_a", &history).unwrap();
        storage.save("session_b", &history).unwrap();

        let sessions = storage.list_sessions().unwrap();
        assert!(sessions.contains(&"session_a".to_string()));
        assert!(sessions.contains(&"session_b".to_string()));

        // Cleanup
        storage.delete("session_a").unwrap();
        storage.delete("session_b").unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_nonexistent_session() {
        let dir = temp_dir();
        let storage = SessionStorage::new(&dir);

        let result = storage.load("nonexistent");
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_delete_session() {
        let dir = temp_dir();
        let storage = SessionStorage::new(&dir);

        let mut history = ConversationHistory::new();
        history.add_user("test");
        storage.save("to_delete", &history).unwrap();

        storage.delete("to_delete").unwrap();
        let result = storage.load("to_delete");
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
