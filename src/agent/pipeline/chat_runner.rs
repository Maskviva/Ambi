// src/agent/pipeline/chat_runner.rs

mod context_handler;
mod multimodal_handler;
mod stream_handler;
mod tool_handler;

use crate::agent::core::Agent;
use crate::error::Result;

use crate::ContentPart;
use std::pin::Pin;
use tokio_stream::wrappers::ReceiverStream;

pub struct ChatRunner;

impl ChatRunner {
    pub async fn chat(agent: &mut Agent, prompt: &str) -> Result<String> {
        Self::chat_multimodal(
            agent,
            vec![ContentPart::Text {
                text: prompt.to_string(),
            }],
        )
        .await
    }

    pub async fn chat_stream(
        agent: &mut Agent,
        prompt: &str,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        Self::chat_multimodal_stream(
            agent,
            vec![ContentPart::Text {
                text: prompt.to_string(),
            }],
        )
        .await
    }

    pub async fn clear_history(agent: &Agent) {
        agent.completion_request.lock().await.chat_history.clear();
        agent.llm_engine.lock().await.reset_context();
    }
}
