use crate::llm::engine::openai_api::openai_api_bridging::OpenAIEngine;
use crate::llm::handler::LLMRequest;
use crate::LLMEngineTrait;
use anyhow::anyhow;
use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc::Sender;

pub mod openai_api_bridging;
pub mod openai_api_config;

#[cfg(feature = "openai-api")]
#[async_trait]
impl LLMEngineTrait for OpenAIEngine {
    async fn chat(&mut self, request: LLMRequest) -> anyhow::Result<String> {
        self.generate_response_sync(request).await.map_err(|e| {
            error!("OpenAI model generation error: {}", e);
            anyhow!("OpenAI error: {}", e)
        })
    }

    async fn chat_stream(&mut self, request: LLMRequest, tx: Sender<Result<String, anyhow::Error>>) {
        if let Err(e) = self.generate_response_stream(request, tx.clone()).await {
            error!("OpenAI stream generation error: {}", e);

            let _ = tx.send(Err(anyhow!("OpenAI API Error: {}", e))).await;
        }
    }

    fn reset_context(&mut self) {
        self.reset_context();
    }
}
