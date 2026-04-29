// src/agent/core.rs

//! Core Agent entities, builders, and memory states.

/// Agent builder implementation for fluent configuration.
pub mod builder;
/// Chat history management and token eviction algorithms.
pub mod history;
/// Prompt compilation and template processing.
pub mod prompt;

use self::history::ChatHistory;
use crate::config::AgentConfig;
use crate::llm::LLMEngine;
use crate::types::{DynTool, Message, StreamFormatter, ToolCallParser, ToolDefinition};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// # Types & Aliases
/// Type alias for a closure that acts as a callback when context tokens are evicted.
#[cfg(not(target_arch = "wasm32"))]
pub type EvictionHandler = Arc<dyn Fn(Vec<Arc<Message>>) + Send + Sync>;
#[cfg(target_arch = "wasm32")]
pub type EvictionHandler = Arc<dyn Fn(Vec<Arc<Message>>)>;

/// Type alias for a factory closure that produces stream formatters per request.
#[cfg(not(target_arch = "wasm32"))]
pub type FormatterFactory = Arc<dyn Fn() -> Box<dyn StreamFormatter + Send + Sync> + Send + Sync>;
#[cfg(target_arch = "wasm32")]
pub type FormatterFactory = Arc<dyn Fn() -> Box<dyn StreamFormatter>>;

// Trait object aliases to handle Send + Sync disparities elegantly.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type DynToolObj = dyn DynTool + Send + Sync;
#[cfg(target_arch = "wasm32")]
pub(crate) type DynToolObj = dyn DynTool;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type ToolCallParserObj = dyn ToolCallParser + Send + Sync;
#[cfg(target_arch = "wasm32")]
pub(crate) type ToolCallParserObj = dyn ToolCallParser;

// --- State Management ---

/// Holds the mutable conversational memory and context of the Agent.
///
/// `AgentState` is decoupled from the `Agent` itself, allowing a single `Agent`
/// instance to handle multiple independent conversations simultaneously.
#[derive(Serialize, Deserialize, Default)]
pub struct AgentState {
    /// The continuous log of the conversation.
    pub chat_history: ChatHistory,
}

impl AgentState {
    /// Creates a fresh, empty conversation state.
    pub fn new() -> Self {
        Self {
            chat_history: ChatHistory::new(),
        }
    }
}

/// # Core Agent Entity
/// The central orchestrator of the LLM pipeline.
///
/// `Agent` acts as a read-only blueprint holding the LLM engine, registered tools,
/// formatting rules, and configurations. Because its internal states are wrapped in `Arc`,
/// cloning an `Agent` is extremely cheap and actively encouraged for high-concurrency setups.
///
/// # Examples
/// ```rust,ignore
/// let agent = Agent::make(engine_config).await?
///     .preamble("You are a helpful assistant.")
///     .tool(WeatherTool)?
///     .with_standard_formatting();
/// ```
#[derive(Clone)]
pub struct Agent {
    // Infrastructure
    pub(crate) llm_engine: Arc<LLMEngine>,

    // Configuration
    pub(crate) config: AgentConfig,

    // Tooling
    pub(crate) tools_def: Arc<Vec<ToolDefinition>>,
    pub(crate) tool_map: Arc<HashMap<String, Arc<DynToolObj>>>,
    pub(crate) tool_parser: Arc<ToolCallParserObj>,
    pub(crate) cached_tool_prompt: String,

    // Processors & Hooks
    pub(crate) formatter_factory: FormatterFactory,
    pub(crate) on_evict_handler: Option<EvictionHandler>,
}

impl Agent {
    /// Evaluates the information entropy (uncertainty) of a specific sentence.
    /// Only works if the underlying engine supports it (e.g., Llama.cpp).
    pub async fn evaluate_sentence_entropy(&self, sentence: &str) -> crate::error::Result<f32> {
        self.llm_engine.evaluate_sentence_entropy(sentence).await
    }
}