// src/llm/providers/openai_api.rs

pub mod config;
mod stream;
mod sync;
/// Request/response translators for OpenAI API compatibility.
pub mod translator;

use self::config::OpenAIEngineConfig;
use crate::error::Result;
use crate::llm::LLMEngineTrait;
use crate::types::LLMRequest;
use async_openai::config::OpenAIConfig;
use async_openai::Client;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

/// The OpenAI API engine implementation.
/// Wraps the async-openai client and provides integration with the Ambi framework.
#[derive(Clone)]
pub struct OpenAIEngine {
    client: Client<OpenAIConfig>,
    cfg: OpenAIEngineConfig,
}

impl OpenAIEngine {
    /// Loads and initializes an OpenAI engine with the given configuration.
    pub fn load(openai_cfg: OpenAIEngineConfig) -> Result<Self> {
        let mut config = OpenAIConfig::new().with_api_key(openai_cfg.api_key.clone());
        config = config.with_api_base(&openai_cfg.base_url);
        let client = Client::with_config(config);
        Ok(Self {
            client,
            cfg: openai_cfg,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl LLMEngineTrait for OpenAIEngine {
    async fn chat(&self, request: LLMRequest) -> Result<String> {
        self.generate_response_sync(request).await
    }

    async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>) {
        if let Err(e) = self.generate_response_stream(request, tx.clone()).await {
            let _ = tx.send(Err(e)).await;
        }
    }

    fn reset_context(&self) {}

    fn supports_multimodal(&self) -> bool {
        true
    }
}
