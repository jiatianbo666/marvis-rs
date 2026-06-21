use async_trait::async_trait;
use marvis_core::{AiResponse, Message, StreamEvent, ToolSchema};

/// The `AiClient` trait abstracts over different AI providers.
///
/// Implementations include:
/// - `DeepSeekClient` for DeepSeek API
/// - `QwenClient` for Qwen Vision API via SiliconFlow
/// - `MockClient` for testing without a real API
#[async_trait]
pub trait AiClient: Send + Sync {
    /// Send a non-streaming chat request.
    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolSchema],
    ) -> Result<AiResponse, marvis_core::MarvisError>;

    /// Send a streaming chat request.
    /// Returns an error if the provider does not support streaming,
    /// or falls back to non-streaming.
    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: &[ToolSchema],
    ) -> Result<Vec<StreamEvent>, marvis_core::MarvisError>;
}
