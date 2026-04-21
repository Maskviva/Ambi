pub mod builder;
pub mod formatter;
pub mod history;
pub mod prompt;

use crate::agent::core::history::ChatHistory;
use crate::agent::tool::{DynTool, ToolCallParser, ToolDefinition};
use crate::llm::{ChatTemplate, LLMEngine};
use crate::types::message::Message;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[derive(Serialize, Deserialize)]
pub struct CompletionRequest {
    pub chat_history: ChatHistory,
    pub __requested: bool,
}

pub struct Agent {
    pub completion_request: Arc<TokioMutex<CompletionRequest>>,
    pub llm_engine: Arc<TokioMutex<LLMEngine>>,
    pub system_prompt: String,
    pub template: ChatTemplate,
    pub tools_def: Arc<Vec<ToolDefinition>>,
    pub tool_map: Arc<HashMap<String, Arc<dyn DynTool>>>,
    pub tool_parser: Arc<dyn ToolCallParser>,
    pub on_evict_handler: Option<Arc<dyn Fn(Vec<Message>) + Send + Sync>>,
    pub max_iterations: usize,
}
