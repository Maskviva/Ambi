// src/llm/engine.rs

//! Core LLM engine abstractions and wrapping mechanisms.

use crate::error::{AmbiError, Result};
use crate::llm::tokenizer::{DefaultTokenizer, TokenizerTrait};
use crate::runtime::SendSync;
use crate::types::LLMRequest;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[cfg(feature = "llama-cpp")]
use crate::llm::providers::llama_cpp::{config::LlamaEngineConfig, LlamaEngine};

#[cfg(feature = "openai-api")]
use crate::llm::providers::openai_api::{config::OpenAIEngineConfig, OpenAIEngine};

/// # Configuration
/// Unified configuration enum for the underlying inference engine.
///
/// Supports switching between different backend implementations via Cargo features.
pub enum LLMEngineConfig {
    /// Configuration for OpenAI-compatible network APIs.
    #[cfg(feature = "openai-api")]
    OpenAI(OpenAIEngineConfig),
    /// Configuration for local Llama.cpp inference.
    #[cfg(feature = "llama-cpp")]
    Llama(LlamaEngineConfig),
    /// Configuration for custom engine.
    Custom(Box<dyn LLMEngineTrait>),
}

/// # Engine Traits
/// The foundational driver contract for LLM engines.
///
/// Any third-party model backend wishing to integrate with the Ambi framework
/// simply needs to implement this Trait and inject it via
/// `Agent::make(LLMEngineConfig::Custom(backend)).await`.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait LLMEngineTrait: SendSync {
    /// Executes a complete synchronous chat inference, returning the full
    /// response string (including raw text for tool calls).
    async fn chat(&self, request: LLMRequest) -> Result<String>;

    /// Executes streaming chat inference.
    ///
    /// Generated text slices must be wrapped in `Ok(String)` and sent via `tx`.
    /// If an error occurs, it should send `Err(AmbiError)` via `tx`, and the
    /// framework will automatically interrupt the stream and clean up resources.
    async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>);

    /// Resets the engine context (e.g., clearing the KV Cache for local inference engines).
    fn reset_context(&self);

    /// Declares whether the engine backend supports multimodal input (image parsing).
    ///
    /// If `false`, the framework will perform a fail-fast interception when
    /// processing inputs containing images.
    fn supports_multimodal(&self) -> bool {
        false
    }

    /// Evaluates the information entropy of a sentence.
    ///
    /// Currently only effective in supported local engines (e.g., Llama-cpp).
    async fn evaluate_sentence_entropy(&self, _sentence: &str) -> Result<f32> {
        Err(AmbiError::EngineError(
            "The current engine backend does not support entropy evaluation.".to_string(),
        ))
    }
}

/// # Engine Wrapper
/// A universal wrapper for LLM engines.
///
/// It aggregates the specific inference `backend` and a `tokenizer`
/// used to calculate context management overhead.
pub struct LLMEngine {
    backend: Box<dyn LLMEngineTrait>,
    /// The active tokenizer utilized for high-speed length estimation.
    pub tokenizer: Arc<dyn TokenizerTrait>,
}

impl LLMEngine {
    /// Loads the corresponding specific underlying engine based on the unified configuration.
    pub fn load(cfg: LLMEngineConfig) -> Result<Self> {
        match cfg {
            #[cfg(feature = "llama-cpp")]
            LLMEngineConfig::Llama(llama_cfg) => {
                llama_cfg.validate()?;
                let engine = LlamaEngine::load(llama_cfg).map_err(|e| {
                    log::error!("Failed to load Llama engine: {}", e);
                    AmbiError::EngineError(format!("Failed to load Llama engine: {}", e))
                })?;

                Ok(LLMEngine {
                    backend: Box::new(engine),
                    tokenizer: Arc::new(DefaultTokenizer::make()?),
                })
            }
            #[cfg(feature = "openai-api")]
            LLMEngineConfig::OpenAI(openai_cfg) => {
                openai_cfg.validate()?;
                let engine = OpenAIEngine::load(openai_cfg).map_err(|e| {
                    log::error!("Failed to load OpenAI engine: {}", e);
                    AmbiError::EngineError(format!("Failed to load OpenAI engine: {}", e))
                })?;

                Ok(LLMEngine {
                    backend: Box::new(engine),
                    tokenizer: Arc::new(DefaultTokenizer::make()?),
                })
            }
            LLMEngineConfig::Custom(backend) => Ok(LLMEngine {
                backend,
                tokenizer: Arc::new(DefaultTokenizer::make()?),
            }),
        }
    }

    /// Injects a custom LLM backend via the `LLMEngineConfig::Custom` variant.
    ///
    /// # Deprecation
    /// This method is deprecated. Use `LLMEngine::load(LLMEngineConfig::Custom(backend))` instead.
    ///
    /// # Migration
    ///
    /// ```rust,ignore
    /// // Old (deprecated):
    /// let engine = LLMEngine::from_custom(Box::new(MyEngine))?;
    ///
    /// // New (recommended):
    /// let engine = LLMEngine::load(LLMEngineConfig::Custom(Box::new(MyEngine)))?;
    /// ```
    #[deprecated(
        since = "0.3.3",
        note = "use `LLMEngine::load(LLMEngineConfig::Custom(backend))` instead"
    )]
    pub fn from_custom(backend: Box<dyn LLMEngineTrait>) -> Result<Self> {
        Ok(Self {
            backend,
            tokenizer: Arc::new(DefaultTokenizer::make()?),
        })
    }

    /// Replaces the framework's default `cl100k_base` tokenizer, injecting an
    /// accurate tokenization algorithm that better matches the specific model.
    pub fn with_custom_tokenizer<T: TokenizerTrait + 'static>(mut self, tokenizer: T) -> Self {
        self.tokenizer = Arc::new(tokenizer);
        self
    }

    /// # Proxy Method Routing
    /// Executes a complete synchronous chat inference, returning the full response string.
    pub async fn chat(&self, request: LLMRequest) -> Result<String> {
        self.backend.chat(request).await
    }

    /// Executes streaming chat inference.
    ///
    /// Generated text slices must be wrapped in `Ok(String)` and sent via `tx`.
    /// If an error occurs, it should send `Err(AmbiError)` via `tx`, and the
    /// framework will automatically interrupt the stream and clean up resources.
    pub async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>) {
        self.backend.chat_stream(request, tx).await
    }

    /// Resets the engine context (e.g., clearing the KV Cache for local inference engines).
    pub fn reset_context(&self) {
        self.backend.reset_context();
    }

    /// Evaluates the information entropy of a sentence.
    ///
    /// Currently only effective in supported local engines (e.g., Llama-cpp).
    pub async fn evaluate_sentence_entropy(&self, sentence: &str) -> Result<f32> {
        self.backend.evaluate_sentence_entropy(sentence).await
    }

    /// Declares whether the engine backend supports multimodal input (image parsing).
    ///
    /// If `false`, the framework will perform a fail-fast interception when
    /// processing inputs containing images.
    pub fn supports_multimodal(&self) -> bool {
        self.backend.supports_multimodal()
    }

    /// Fast, purely synchronous token calculation to support the Agent's memory eviction algorithm.
    pub fn count_tokens(&self, text: &str) -> Result<usize> {
        self.tokenizer.count_tokens(text)
    }
}
