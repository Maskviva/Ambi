use crate::agent::Message;
use crate::llm::engine::{LlamaEngine, OpenAIEngine};
use crate::EngineConfig;
use anyhow::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct LLMRequest {
    pub system_prompt: String,
    pub history: Vec<Message>,
    pub tool_prompt: String,
    pub formatted_prompt: String,
}

#[async_trait]
pub trait LLMEngineTrait: Send + Sync {
    async fn chat(&mut self, request: LLMRequest) -> Result<String>;
    async fn chat_stream(&mut self, request: LLMRequest, tx: Sender<String>);
    fn reset_context(&mut self);
}

pub struct LLMEngine {
    backend: Box<dyn LLMEngineTrait>,
}

impl LLMEngine {
    pub fn load(cfg: EngineConfig) -> Result<Self> {
        match cfg {
            #[cfg(feature = "llama-cpp")]
            EngineConfig::Llama(llama_cfg) => {
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
            EngineConfig::OpenAI(openai_cfg) => {
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

    pub async fn chat_stream(&mut self, request: LLMRequest, tx: Sender<String>) {
        self.backend.chat_stream(request, tx).await
    }

    pub fn reset_context(&mut self) {
        self.backend.reset_context();
    }
}
