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
pub type EvictionHandler = Arc<dyn Fn(&AgentState, Vec<Arc<Message>>) + Send + Sync>;
/// Type alias for a closure that acts as a callback when context tokens are evicted.
#[cfg(target_arch = "wasm32")]
pub type EvictionHandler = Arc<dyn Fn(&AgentState, Vec<Arc<Message>>)>;

/// Type alias for a factory closure that produces stream formatters per request.
#[cfg(not(target_arch = "wasm32"))]
pub type FormatterFactory = Arc<dyn Fn() -> Box<dyn StreamFormatter + Send + Sync> + Send + Sync>;
/// Type alias for a factory closure that produces stream formatters per request.
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

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type ExtensionsMap = anymap2::SendSyncAnyMap;
#[cfg(target_arch = "wasm32")]
pub(crate) type ExtensionsMap = anymap2::AnyMap;

// --- State Management ---

/// Holds the mutable conversational memory and context of the Agent.
///
/// `AgentState` is decoupled from the `Agent` itself, allowing a single `Agent`
/// instance to handle multiple independent conversations simultaneously.
#[derive(Serialize, Deserialize, Default)]
pub struct AgentState {
    /// A unique identifier for the current conversation session.
    /// Useful for KV cache slotting in local engines or distributed tracing in cloud APIs.
    pub session_id: String,

    /// Contextual background information that changes dynamically during the session
    /// (e.g., RAG results, current time). Safely prepended to the system prompt without
    /// being affected by token eviction.
    pub dynamic_context: String,

    /// The continuous log of the conversation.
    pub chat_history: ChatHistory,

    /// Extensions for custom state management.
    #[serde(skip)]
    extensions: ExtensionsMap,
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
    pub(crate) config: Arc<AgentConfig>,

    // Tooling
    pub(crate) tools_def: Arc<Vec<ToolDefinition>>,
    pub(crate) tool_map: Arc<HashMap<String, Arc<DynToolObj>>>,
    pub(crate) tool_parser: Arc<ToolCallParserObj>,

    pub(crate) cached_tool_prompt: Arc<String>,

    // Processors
    pub(crate) formatter_factory: FormatterFactory,

    // Hooks
    pub(crate) on_evict_handler: Option<EvictionHandler>,
}

impl AgentState {
    /// Clears the chat history and resets the engine context.
    pub fn clear_history(&mut self, agent: &Agent) {
        self.chat_history.clear();
        agent.llm_engine.reset_context();
    }

    /// Convenience helper: Directly sets or overwrites the dynamic context.
    /// Typically used to inject fresh Retrieval-Augmented Generation (RAG) results or environment states.
    pub fn set_dynamic_context(&mut self, context: &str) {
        self.dynamic_context = context.to_string();
    }

    /// Convenience helper: Appends content to the existing dynamic context, automatically handling newline separators.
    /// Useful for stacking background knowledge from multiple external modules (e.g., combining time info and RAG results).
    pub fn append_dynamic_context(&mut self, context: &str) {
        if context.is_empty() {
            return;
        }
        if !self.dynamic_context.is_empty() {
            self.dynamic_context.push_str("\n\n");
        }
        self.dynamic_context.push_str(context);
    }

    /// Convenience helper: Clears the dynamic context entirely.
    pub fn clear_dynamic_context(&mut self) {
        self.dynamic_context.clear();
    }
}

impl Agent {
    /// Evaluates the information entropy (uncertainty) of a specific sentence.
    /// Only works if the underlying engine supports it (e.g., Llama.cpp).
    pub async fn evaluate_sentence_entropy(&self, sentence: &str) -> crate::error::Result<f32> {
        self.llm_engine.evaluate_sentence_entropy(sentence).await
    }

    /// Returns a cloned `Arc` to the underlying [`LLMEngine`].
    ///
    /// This provides direct access to the engine (e.g., for custom inference calls,
    /// parameter tweaking, or engine-specific operations). Cloning the `Arc` is
    /// cheap and does not duplicate the engine itself.
    ///
    /// # Note
    ///
    /// Mutations performed on the engine may affect all `Agent` instances sharing
    /// the same engine handle.
    pub fn get_llama_engine(&self) -> Arc<LLMEngine> {
        self.llm_engine.clone()
    }
}
