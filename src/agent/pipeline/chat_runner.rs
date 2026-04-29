// src/agent/pipeline/chat_runner.rs

//! The primary pipeline implementation orchestrating ReAct interactions.
mod context_handler;
mod multimodal_handler;
mod stream_handler;
mod tool_handler;

use crate::agent::core::{Agent, AgentState};
use crate::agent::pipeline::Pipeline;
use crate::error::{AmbiError, Result};
use crate::ContentPart;

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

pub(crate) struct StateManager<'a>(&'a Arc<RwLock<AgentState>>);

/// The default chat runner implementing the full LLM-Tooling event loop.
pub struct ChatRunner;

impl Pipeline for ChatRunner {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        Self::validate_multimodal_input(agent, &input)?;
        Self::chat_multimodal(agent, state, input).await
    }

    async fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        Self::validate_multimodal_input(agent, &input)?;
        Self::chat_multimodal_stream(agent, state, input).await
    }
}

impl ChatRunner {
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
    fn validate_multimodal_input(agent: &Agent, parts: &[ContentPart]) -> Result<()> {
        let has_image = parts.iter().any(|p| matches!(p, ContentPart::Image { .. }));
        if has_image && !agent.llm_engine.supports_multimodal() {
            Err(AmbiError::EngineError(
                "Security Check Failed: The current LLM engine does not support multimodal (image) inputs.".into()
            ))
        } else {
            Ok(())
        }
    }
}
