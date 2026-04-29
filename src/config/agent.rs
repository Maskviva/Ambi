// src/config/agent.rs

use crate::types::ChatTemplate;
use serde::{Deserialize, Serialize};

/// Defines the policy for truncating conversation history when approaching token limits.
///
/// The framework uses a strict First-In-First-Out (FIFO) algorithm to pop the oldest
/// conversation logs until the total context token count safely falls below `max_safe_tokens`.
///
/// # Examples
/// ```rust
/// use ambi::config::EvictionStrategy;
///
/// let strategy = EvictionStrategy {
///     max_safe_tokens: 4096, // E.g., for an 8K model, leaving ample room for outputs
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvictionStrategy {
    /// The maximum allowable token count for the conversational history.
    /// Exceeding this value will trigger an automatic historical eviction.
    pub max_safe_tokens: usize,
}

impl Default for EvictionStrategy {
    fn default() -> Self {
        Self {
            max_safe_tokens: 8000,
        }
    }
}

/// Top-level configuration properties for the Agent.
#[derive(Clone, Debug)]
pub struct AgentConfig {
    /// The master directive informing the Agent of its persona and constraints.
    pub system_prompt: String,

    /// The templating engine defining how chat histories map into plain string sequences.
    pub template: ChatTemplate,

    /// The absolute maximum number of back-and-forth internal iterations (LLM -> Tool -> LLM)
    /// allowed per single user request before forcefully halting.
    pub max_iterations: usize,

    /// The strategy used to prune context limits.
    pub eviction_strategy: EvictionStrategy,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            template: ChatTemplate::chatml(),
            max_iterations: 10,
            eviction_strategy: EvictionStrategy::default(),
        }
    }
}
