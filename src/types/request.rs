// src/types/request.rs

//! Communication payload formatting for the LLM engine.

use super::{Message, ToolDefinition};
use std::sync::Arc;

/// The standard payload compiled by the Agent and sent to the LLM engine.
///
/// # Session identity
///
/// `LLMRequest` is per-request, but a single engine handle is typically shared by many
/// conversation states (and, in `ChatRunner`, by concurrent tool calls). `session_id` is the
/// identity of the [`crate::AgentState`] that produced this request, so a stateful backend can
/// keep one cache slot per conversation instead of assuming a global one.
///
/// Stateless backends (the OpenAI-compatible engine) may ignore it. Backends that keep decoder
/// state must treat a change of `session_id` as "the cache belongs to someone else".
#[derive(Clone, Debug, serde::Serialize)]
pub struct LLMRequest {
    /// Identity of the originating conversation, used to scope stateful backend caches.
    pub session_id: String,
    /// The aggregated system preamble.
    pub system_prompt: String,
    /// The filtered list of conversational history.
    pub history: Vec<Arc<Message>>,
    /// The list of available tool definitions provided to the LLM.
    pub tools: Vec<ToolDefinition>,
    /// The raw tool calling instructions injected into the prompt.
    pub tool_prompt: String,
    /// The fully rendered, raw prompt string (used primarily by local engines).
    pub formatted_prompt: String,
    /// The tags used to identify tool calls (start_tag, end_tag).
    pub tool_tags: (String, String),
    /// A collection of base64 extracted images for multimodal inference.
    pub images: Vec<String>,
}
