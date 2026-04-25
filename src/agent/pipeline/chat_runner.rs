// src/agent/pipeline/chat_runner.rs

mod context_handler;
mod multimodal_handler;
mod stream_handler;
mod tool_handler;

use crate::agent::core::{Agent, AgentState};
use crate::agent::pipeline::Pipeline;
use crate::error::Result;
use crate::ContentPart;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use tokio_stream::wrappers::ReceiverStream;

pub(crate) struct StateManager<'a>(&'a Arc<StdMutex<AgentState>>);

pub struct ChatRunner;

impl Pipeline for ChatRunner {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<StdMutex<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        Self::chat_multimodal(agent, state, input).await
    }

    async fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<StdMutex<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        Self::chat_multimodal_stream(agent, state, input).await
    }
}

impl ChatRunner {
    pub async fn chat(
        &self,
        agent: &Agent,
        state: &Arc<StdMutex<AgentState>>,
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

    pub async fn chat_stream(
        &self,
        agent: &Agent,
        state: &Arc<StdMutex<AgentState>>,
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

    pub fn clear_history(agent: &Agent, state: &mut AgentState) {
        state.chat_history.clear();
        agent.llm_engine.reset_context();
    }
}
