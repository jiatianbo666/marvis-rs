//! Context window management.

use crate::history::ConversationHistory;

/// Manages the AI context window, ensuring conversations stay within token limits.
pub struct ContextManager {
    max_tokens: usize,
    system_prompt: Option<String>,
}

impl ContextManager {
    /// Create a new context manager with the given token limit.
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            system_prompt: None,
        }
    }

    /// Set the system prompt.
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }

    /// Get the system prompt.
    pub fn system_prompt(&self) -> Option<&str> {
        self.system_prompt.as_deref()
    }

    /// Get the max token limit.
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }

    /// Set a new max token limit.
    pub fn set_max_tokens(&mut self, max: usize) {
        self.max_tokens = max;
    }

    /// Estimate token count for a string (rough: ~4 chars per token).
    pub fn estimate_tokens(text: &str) -> usize {
        text.chars().count() / 4
    }

    /// Check if the history fits within the token budget.
    pub fn fits_budget(&self, history: &ConversationHistory) -> bool {
        history.estimated_tokens() <= self.max_tokens
    }

    /// Trim the history to fit the budget.
    pub fn trim(&self, history: &mut ConversationHistory) {
        history.trim_to_budget(self.max_tokens);
    }

    /// Build a full message list for the AI API from the history,
    /// adding the system prompt if present.
    pub fn build_messages(&self, history: &ConversationHistory) -> Vec<marvis_core::Message> {
        let mut messages = history.messages().to_vec();

        // Ensure system prompt is at the beginning
        if let Some(ref prompt) = self.system_prompt {
            let has_system = messages
                .first()
                .map(|m| matches!(m.role, marvis_core::Role::System))
                .unwrap_or(false);

            if !has_system {
                messages.insert(0, marvis_core::Message::system(prompt.as_str()));
            } else if let Some(first) = messages.first_mut() {
                *first = marvis_core::Message::system(prompt.as_str());
            }
        }

        messages
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(8000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context_manager() {
        let cm = ContextManager::new(4000);
        assert_eq!(cm.max_tokens(), 4000);
        assert!(cm.system_prompt().is_none());
    }

    #[test]
    fn test_set_system_prompt() {
        let mut cm = ContextManager::new(8000);
        cm.set_system_prompt("You are helpful");
        assert_eq!(cm.system_prompt(), Some("You are helpful"));
    }

    #[test]
    fn test_estimate_tokens() {
        // Roughly 4 chars per token
        let tokens = ContextManager::estimate_tokens("hello world! hello world!");
        assert_eq!(tokens, 6); // 24 chars / 4
    }

    #[test]
    fn test_fits_budget() {
        let cm = ContextManager::new(1000);
        let mut history = ConversationHistory::new();
        history.add_user("short");
        assert!(cm.fits_budget(&history));

        let mut long = ConversationHistory::new();
        long.add_user("x".repeat(10000));
        assert!(!cm.fits_budget(&long));
    }

    #[test]
    fn test_build_messages_adds_system_prompt() {
        let mut cm = ContextManager::new(8000);
        cm.set_system_prompt("You are helpful");
        let history = ConversationHistory::new();

        let messages = cm.build_messages(&history);
        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0].role, marvis_core::Role::System));
        assert_eq!(messages[0].content, "You are helpful");
    }

    #[test]
    fn test_build_messages_preserves_existing() {
        let cm = ContextManager::new(8000);
        let mut history = ConversationHistory::new();
        history.add_user("hello");
        history.add_assistant("hi", None);

        let messages = cm.build_messages(&history);
        assert!(messages.len() >= 2);
    }
}
