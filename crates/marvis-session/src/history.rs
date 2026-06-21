//! Conversation history management.

use marvis_core::Message;
use serde::{Deserialize, Serialize};

/// Manages the conversation history between the user and the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    messages: Vec<Message>,
}

impl ConversationHistory {
    /// Create a new empty conversation history.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Create a new history with a system prompt.
    pub fn with_system_prompt(system_prompt: impl Into<String>) -> Self {
        Self {
            messages: vec![Message::system(system_prompt)],
        }
    }

    /// Add a message to the history.
    pub fn add(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Add a system message.
    pub fn add_system(&mut self, content: impl Into<String>) {
        self.messages.push(Message::system(content));
    }

    /// Add a user message.
    pub fn add_user(&mut self, content: impl Into<String>) {
        self.messages.push(Message::user(content));
    }

    /// Add an assistant message.
    pub fn add_assistant(
        &mut self,
        content: impl Into<String>,
        tool_calls: Option<Vec<marvis_core::ToolCall>>,
    ) {
        self.messages.push(Message::assistant(content, tool_calls));
    }

    /// Add a tool result message.
    pub fn add_tool_result(
        &mut self,
        tool_call_id: impl Into<String>,
        content: impl Into<String>,
        tool_name: Option<String>,
    ) {
        self.messages
            .push(Message::tool_result(tool_call_id, content, tool_name));
    }

    /// Get all messages.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Get a mutable reference to all messages.
    pub fn messages_mut(&mut self) -> &mut Vec<Message> {
        &mut self.messages
    }

    /// Number of messages in the history.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether the history is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Clear all messages.
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Estimate the total token count (rough estimate: ~4 chars per token).
    pub fn estimated_tokens(&self) -> usize {
        self.messages.iter().map(|m| m.content.len() / 4).sum()
    }

    /// Trim the history to fit within a token budget, keeping the system prompt
    /// and the most recent messages.
    pub fn trim_to_budget(&mut self, max_tokens: usize) {
        let system_idx = self
            .messages
            .iter()
            .position(|m| matches!(m.role, marvis_core::Role::System));

        let system_msg = system_idx.map(|i| self.messages.remove(i));

        // Remove oldest non-system messages until we fit
        while self.estimated_tokens() > max_tokens && self.len() > 2 {
            self.messages.remove(0);
        }

        // Put system prompt back at the beginning
        if let Some(sys) = system_msg {
            self.messages.insert(0, sys);
        }
    }

    /// Serialize history to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.messages)
    }

    /// Deserialize history from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let messages: Vec<Message> = serde_json::from_str(json)?;
        Ok(Self { messages })
    }
}

impl Default for ConversationHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history_is_empty() {
        let history = ConversationHistory::new();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_with_system_prompt() {
        let history = ConversationHistory::with_system_prompt("You are helpful");
        assert_eq!(history.len(), 1);
        assert!(matches!(
            history.messages()[0].role,
            marvis_core::Role::System
        ));
    }

    #[test]
    fn test_add_messages() {
        let mut history = ConversationHistory::new();
        history.add_user("hello");
        history.add_assistant("hi there", None);
        assert_eq!(history.len(), 2);
        assert!(matches!(
            history.messages()[0].role,
            marvis_core::Role::User
        ));
        assert!(matches!(
            history.messages()[1].role,
            marvis_core::Role::Assistant
        ));
    }

    #[test]
    fn test_add_tool_result() {
        let mut history = ConversationHistory::new();
        history.add_tool_result("call_1", "result", Some("read_file".to_string()));
        assert_eq!(history.len(), 1);
        let msg = &history.messages()[0];
        assert!(matches!(msg.role, marvis_core::Role::Tool));
        assert_eq!(msg.tool_call_id, Some("call_1".to_string()));
        assert_eq!(msg.name, Some("read_file".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut history = ConversationHistory::new();
        history.add_user("hello");
        history.add_assistant("hi", None);
        history.clear();
        assert!(history.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut history = ConversationHistory::new();
        history.add_system("system");
        history.add_user("user");
        history.add_assistant("assistant", None);

        let json = history.to_json().unwrap();
        let restored = ConversationHistory::from_json(&json).unwrap();
        assert_eq!(restored.len(), 3);
        assert_eq!(restored.messages()[1].content, "user");
    }

    #[test]
    fn test_estimated_tokens() {
        let mut history = ConversationHistory::new();
        history.add_user("hello world"); // ~2-3 tokens
        assert!(history.estimated_tokens() > 0);
    }

    #[test]
    fn test_trim_to_budget_preserves_recent() {
        let mut history = ConversationHistory::new();
        for i in 0..100 {
            history.add_user(format!("message {}", i));
        }
        let original_len = history.len();
        history.trim_to_budget(50); // very small budget
        assert!(history.len() < original_len);
    }

    #[test]
    fn test_iter() {
        let mut history = ConversationHistory::new();
        history.add_user("a");
        history.add_user("b");
        let items: Vec<_> = history.iter().collect();
        assert_eq!(items.len(), 2);
    }
}

/// An iterator over conversation messages.
pub struct HistoryIterator<'a> {
    messages: &'a [Message],
    index: usize,
}

impl<'a> Iterator for HistoryIterator<'a> {
    type Item = &'a Message;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.messages.len() {
            let item = &self.messages[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl ConversationHistory {
    /// Iterate over messages by reference.
    pub fn iter(&self) -> HistoryIterator<'_> {
        HistoryIterator {
            messages: &self.messages,
            index: 0,
        }
    }
}
