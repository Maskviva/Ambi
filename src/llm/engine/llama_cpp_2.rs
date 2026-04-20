use crate::llm::engine::llama_cpp_2::llama_cpp_2_bridging::LlamaEngine;
use crate::llm::handler::LLMRequest;
use crate::LLMEngineTrait;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

pub mod llama_cpp_2_bridging;
pub mod llama_cpp_2_config;

#[cfg(feature = "llama-cpp")]
#[async_trait]
impl LLMEngineTrait for LlamaEngine {
    async fn chat(&mut self, request: LLMRequest) -> anyhow::Result<String> {
        self.chat_internal(&request.formatted_prompt).await
    }

    async fn chat_stream(&mut self, request: LLMRequest, tx: Sender<String>) {
        self.stream_internal(&request.formatted_prompt, tx).await;
    }

    fn reset_context(&mut self) {
        self.reset_internal();
    }
}
