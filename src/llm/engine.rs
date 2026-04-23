// src/llm/engine.rs
use crate::error::{AmbiError, Result};
use crate::llm::LLMEngineConfig;
use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc::Sender;

#[cfg(feature = "llama-cpp")]
use crate::llm::providers::llama_cpp::LlamaEngine;

#[cfg(feature = "openai-api")]
use crate::llm::providers::openai_api::OpenAIEngine;
use crate::types::LLMRequest;

#[async_trait]
pub trait LLMEngineTrait: Send + Sync {
    async fn chat(&mut self, request: LLMRequest) -> Result<String>;
    async fn chat_stream(&mut self, request: LLMRequest, tx: Sender<Result<String>>);
    fn reset_context(&mut self);

    async fn evaluate_sentence_entropy(&mut self, _sentence: &str) -> Result<f32> {
        Err(AmbiError::EngineError(
            "The current engine backend does not support entropy evaluation. Local Llama-cpp engine is required.".to_string()
        ))
    }
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
                    AmbiError::EngineError(format!("Failed to load Llama engine: {}", e))
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
                    AmbiError::EngineError(format!("Failed to load OpenAI engine: {}", e))
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

    pub async fn chat_stream(&mut self, request: LLMRequest, tx: Sender<Result<String>>) {
        self.backend.chat_stream(request, tx).await
    }

    pub fn reset_context(&mut self) {
        self.backend.reset_context();
    }

    pub async fn evaluate_sentence_entropy(&mut self, sentence: &str) -> Result<f32> {
        self.backend.evaluate_sentence_entropy(sentence).await
    }
}
