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
/// async fn main() -> Result<()> {
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
        let mut engine = self.llm_engine.lock().await;
        engine.evaluate_sentence_entropy(sentence).await
    }
}
