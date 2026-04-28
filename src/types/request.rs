// src/types/request.rs

use crate::agent::ToolDefinition;
use crate::types::message::Message;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct LLMRequest {
    pub system_prompt: String,
    pub history: Vec<Arc<Message>>,
    pub tools: Vec<ToolDefinition>,
    pub tool_prompt: String,
    pub formatted_prompt: String,
    pub tool_tags: (String, String),
    pub images: Vec<String>,
}
