use std::sync::Arc;
// src/llm/engine.rs
use crate::error::{AmbiError, Result};
use crate::llm::tokenizer::{DefaultTokenizer, TokenizerTrait};
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
    async fn chat(&self, request: LLMRequest) -> Result<String>;
    async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>);
    fn reset_context(&self);

    fn supports_multimodal(&self) -> bool {
        false
    }

    async fn evaluate_sentence_entropy(&self, _sentence: &str) -> Result<f32> {
        Err(AmbiError::EngineError(
            "The current engine backend does not support entropy evaluation. Local Llama-cpp engine is required.".to_string()
        ))
    }
}

pub struct LLMEngine {
    backend: Box<dyn LLMEngineTrait>,
    pub tokenizer: Arc<dyn TokenizerTrait>,
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
                    tokenizer: Arc::new(DefaultTokenizer::make()),
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
                    tokenizer: Arc::new(DefaultTokenizer::make()),
                })
            }
        }
    }

    pub fn from_custom(backend: Box<dyn LLMEngineTrait>) -> Self {
        Self {
            backend,
            tokenizer: Arc::new(DefaultTokenizer::make()),
        }
    }

    pub fn with_custom_tokenizer<T: TokenizerTrait + 'static>(mut self, tokenizer: T) -> Self {
        self.tokenizer = Arc::new(tokenizer);
        self
    }

    pub async fn chat(&self, request: LLMRequest) -> Result<String> {
        self.backend.chat(request).await
    }
    pub async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>) {
        self.backend.chat_stream(request, tx).await
    }
    pub fn reset_context(&self) {
        self.backend.reset_context();
    }
    pub async fn evaluate_sentence_entropy(&self, sentence: &str) -> Result<f32> {
        self.backend.evaluate_sentence_entropy(sentence).await
    }

    pub fn count_tokens(&self, text: &str) -> usize {
        self.tokenizer.count_tokens(text).unwrap_or(0)
    }

    pub fn supports_multimodal(&self) -> bool {
        self.backend.supports_multimodal()
    }
}
