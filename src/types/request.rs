// src/types/request.rs

//! Communication payload formatting for the LLM engine.

use super::{Message, ToolDefinition};
use std::sync::Arc;

/// The standard payload compiled by the Agent and sent to the LLM engine.
#[derive(Clone, Debug)]
pub struct LLMRequest {
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
