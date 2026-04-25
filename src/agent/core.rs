// src/agent/core.rs
pub mod builder;
pub mod context;
pub mod formatter;
pub mod prompt;

use crate::agent::core::context::ChatHistory;
use crate::agent::tool::{DynTool, ToolCallParser, ToolDefinition};
use crate::llm::LLMEngine;
use crate::types::config::AgentConfig;
use crate::types::message::Message;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub type EvictionHandler = Arc<dyn Fn(Vec<Arc<Message>>) + Send + Sync>;

#[derive(Serialize, Deserialize, Default)]
pub struct AgentState {
    pub chat_history: ChatHistory,
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            chat_history: ChatHistory::new(),
        }
    }
}

#[derive(Clone)]
pub struct Agent {
    pub llm_engine: Arc<LLMEngine>,
    pub config: AgentConfig,
    pub tools_def: Arc<Vec<ToolDefinition>>,
    pub tool_map: Arc<HashMap<String, Arc<dyn DynTool>>>,
    pub tool_parser: Arc<dyn ToolCallParser>,
    pub on_evict_handler: Option<EvictionHandler>,
    pub cached_tool_prompt: String,
}

impl Agent {
    #[cfg(feature = "llama-cpp")]
    pub async fn evaluate_sentence_entropy(&self, sentence: &str) -> crate::error::Result<f32> {
        self.llm_engine.evaluate_sentence_entropy(sentence).await
    }
}
