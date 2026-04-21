use crate::llm::{LLMEngineConfig, LLMRequest};
use anyhow::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc::Sender;

#[cfg(feature = "llama-cpp")]
use crate::llm::providers::llama_cpp::LlamaEngine;

#[cfg(feature = "openai-api")]
use crate::llm::providers::openai_api::OpenAIEngine;

#[async_trait]
pub trait LLMEngineTrait: Send + Sync {
    async fn chat(&mut self, request: LLMRequest) -> Result<String>;
    async fn chat_stream(&mut self, request: LLMRequest, tx: Sender<Result<String, anyhow::Error>>);
    fn reset_context(&mut self);
}

pub struct LLMEngine {
    backend: Box<dyn LLMEngineTrait>,
}

impl LLMEngine {
    pub fn load(cfg: LLMEngineConfig) -> Result<Self> {
        match cfg {
            #[cfg(feature = "llama-cpp")]
            LLMEngineConfig::Llama(llama_cfg) => {
                llama_cfg.validate()?;
                let engine = LlamaEngine::load(llama_cfg).map_err(|e| {
                    error!("Failed to load Llama engine: {}", e);
                    anyhow::anyhow!("Failed to load Llama engine: {}", e)
                })?;
                Ok(LLMEngine {
                    backend: Box::new(engine),
                })
            }
            #[cfg(feature = "openai-api")]
            LLMEngineConfig::OpenAI(openai_cfg) => {
                openai_cfg.validate()?;
                let engine = OpenAIEngine::load(openai_cfg).map_err(|e| {
                    error!("Failed to load OpenAI engine: {}", e);
                    anyhow::anyhow!("Failed to load OpenAI engine: {}", e)
                })?;
                Ok(LLMEngine {
                    backend: Box::new(engine),
                })
            }
        }
    }

    pub fn from_custom(backend: Box<dyn LLMEngineTrait>) -> Self {
        Self { backend }
    }

    pub async fn chat(&mut self, request: LLMRequest) -> Result<String> {
        self.backend.chat(request).await
    }

    pub async fn chat_stream(
        &mut self,
        request: LLMRequest,
        tx: Sender<Result<String, anyhow::Error>>,
    ) {
        self.backend.chat_stream(request, tx).await
    }

    pub fn reset_context(&mut self) {
        self.backend.reset_context();
    }
}
