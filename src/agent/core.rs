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
}

pub type EvictionHandler = Arc<dyn Fn(Vec<Arc<Message>>) + Send + Sync>;

/// The core orchestration unit of the Ambi framework.
///
/// The `Agent` is responsible for managing the conversation state, interacting with
/// the underlying LLM engine, parsing tool calls, and maintaining the prompt lifecycle.
/// It acts as the bridge between user inputs, model generations, and local tool executions.
///
/// # Examples
///
/// ```rust
/// use ambi::{Agent, LLMEngineConfig};
/// use ambi::llm::providers::openai_api::OpenAIEngineConfig;
/// use ambi::llm::ChatTemplateType;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = LLMEngineConfig::OpenAI(OpenAIEngineConfig {
///         api_key: "your-api-key".to_string(),
///         base_url: "[https://api.openai.com/v1](https://api.openai.com/v1)".to_string(),
///         model_name: "gpt-4o-mini".to_string(),
///         temp: 0.7,
///         top_p: 0.9,
///     });
///
///     let mut agent = Agent::make(config).await?
///         .preamble("You are a helpful AI assistant.")
///         .template(ChatTemplateType::Chatml);
///     
///     Ok(())
/// }
/// ```
pub struct Agent {
    pub completion_request: Arc<TokioMutex<CompletionRequest>>,
    pub llm_engine: Arc<TokioMutex<LLMEngine>>,
    pub system_prompt: String,
    pub template: ChatTemplate,
    pub tools_def: Arc<Vec<ToolDefinition>>,
    pub tool_map: Arc<HashMap<String, Arc<dyn DynTool>>>,
    pub tool_parser: Arc<dyn ToolCallParser>,
    pub on_evict_handler: Option<EvictionHandler>,
    pub max_iterations: usize,
    pub enable_formatting: bool,
    pub eviction_strategy: (usize, usize, usize),
    pub cached_tool_prompt: String,
}
