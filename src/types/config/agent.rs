// src/types/config/agent.rs

use crate::llm::ChatTemplate;
use serde::{Deserialize, Serialize};

/// # Note on Token Limits
/// The default `max_safe_tokens` is 8000. For smaller models (e.g., 2048 context),
/// set this to a lower value to avoid context overflow. For very large contexts,
/// you may increase it further.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvictionStrategy {
    /// Number of messages retained at the beginning of the conversation
    pub keep_head: usize,
    /// Number of messages retained at the end of the conversation
    pub keep_tail: usize,
    /// Maximum safe number of tokens (exceeding this value will trigger eviction)
    pub max_safe_tokens: usize,
}

impl Default for EvictionStrategy {
    fn default() -> Self {
        Self {
            keep_head: 2,
            keep_tail: 6,
            max_safe_tokens: 8000,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AgentConfig {
    pub system_prompt: String,
    pub template: ChatTemplate,
    pub max_iterations: usize,
    pub enable_formatting: bool,
    /// Context eviction policy
    pub eviction_strategy: EvictionStrategy,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            template: ChatTemplate::chatml(),
            max_iterations: 10,
            enable_formatting: false,
            eviction_strategy: EvictionStrategy::default(),
        }
    }
}
