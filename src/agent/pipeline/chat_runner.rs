// src/agent/pipeline/chat_runner.rs

//! The primary pipeline implementation orchestrating ReAct interactions.
mod context_handler;
#[cfg(feature = "sub-agent")]
pub mod leaf;
mod multimodal_handler;
mod stream_handler;
mod tool_handler;

use crate::agent::core::{Agent, AgentState};
use crate::agent::pipeline::Pipeline;
use crate::error::{AmbiError, Result};
use crate::ContentPart;

#[cfg(feature = "sub-agent")]
pub use leaf::{LeafConfig, DEFAULT_LEAF_TIMEOUT_SECS, DEFAULT_LEAF_TOOL_NAME};

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

pub(crate) struct StateManager<'a>(&'a Arc<RwLock<AgentState>>);

/// The default chat runner implementing the full LLM-Tooling event loop.
#[derive(Clone)]
pub struct ChatRunner {
    /// The maximum number of concurrent requests allowed.
    pub maximum_concurrency: usize,

    /// Leaf sub-agent configuration. `None` unless explicitly enabled via
    /// [`ChatRunner::with_leaf`] or [`ChatRunner::with_leaf_config`]. Kept private so the only way
    /// to turn the capability on is the validating builder; only present with the `sub-agent` feature.
    #[cfg(feature = "sub-agent")]
    leaf_config: Option<LeafConfig>,
}

impl Pipeline for ChatRunner {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        self.validate_multimodal_input(agent, &input)?;
        self.chat_multimodal(agent, state, input).await
    }

    async fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        self.validate_multimodal_input(agent, &input)?;
        self.chat_multimodal_stream(agent, state, input).await
    }
}

impl Default for ChatRunner {
    fn default() -> Self {
        Self::new(5)
    }
}

impl ChatRunner {
    /// Creates a new chat runner
    pub fn new(maximum_concurrency: usize) -> Self {
        Self {
            maximum_concurrency,
            #[cfg(feature = "sub-agent")]
            leaf_config: None,
        }
    }

    /// Enables the leaf sub-agent tool (`spawn_leaf`) on this [`ChatRunner`] using [`LeafConfig::default`].
    ///
    /// Enabling the `sub-agent` Cargo feature alone leaves [`ChatRunner::default`] unchanged;
    /// call this builder method (or [`ChatRunner::with_leaf_config`]) to opt in at runtime.
    #[cfg(feature = "sub-agent")]
    pub fn with_leaf(mut self) -> Self {
        self.leaf_config = Some(LeafConfig::default());
        self
    }

    /// Enables the leaf sub-agent tool on this [`ChatRunner`] with a custom [`LeafConfig`].
    #[cfg(feature = "sub-agent")]
    pub fn with_leaf_config(mut self, config: LeafConfig) -> Self {
        self.leaf_config = Some(config);
        self
    }

    /// # Public Helpers
    /// Synchronous chat execution helper for pure text prompts.
    pub async fn chat(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        prompt: &str,
    ) -> Result<String> {
        self.execute(
            agent,
            state,
            vec![ContentPart::Text {
                text: prompt.to_string(),
            }],
        )
        .await
    }

    /// Resets the agent's contextual memory entirely.
    pub async fn chat_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        prompt: &str,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        self.execute_stream(
            agent,
            state,
            vec![ContentPart::Text {
                text: prompt.to_string(),
            }],
        )
        .await
    }

    /// Resets the agent's contextual memory entirely.
    pub fn clear_history(agent: &Agent, state: &mut AgentState) {
        state.chat_history.clear();
        agent.llm_engine.reset_context();
    }

    /// # Internal Validators
    fn validate_multimodal_input(&self, agent: &Agent, parts: &[ContentPart]) -> Result<()> {
        if self.maximum_concurrency == 0 {
            return Err(AmbiError::PipelineError(
                "Maximum concurrency must be greater than 0".into(),
            ));
        }

        let has_image = parts.iter().any(|p| matches!(p, ContentPart::Image { .. }));
        if has_image && !agent.llm_engine.supports_multimodal() {
            Err(AmbiError::engine_kind(
                crate::error::EngineErrorKind::InvalidRequest,
                "Security Check Failed: The current LLM engine does not support multimodal (image) inputs.".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}
