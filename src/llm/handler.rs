#[cfg(feature = "local")]
use crate::llm::engine::LlamaEngine;

#[cfg(feature = "cloud")]
use crate::llm::openai::OpenAIEngine;

use crate::llm::{EngineBackend, EngineConfig};
use anyhow::{anyhow, Result};
use log::error;
use std::cell::RefCell;
use tokio::sync::mpsc::UnboundedSender;

pub struct LLMEngine {
    backend: EngineBackend,
}

impl LLMEngine {
    pub fn load(cfg: EngineConfig) -> Result<Self> {
        #[cfg(feature = "cloud")]
        if let Some(openai_cfg) = cfg.open_ai {
            let engine = OpenAIEngine::load(openai_cfg).map_err(|e| {
                error!("Failed to load OpenAI engine: {}", e);
                anyhow::anyhow!("Failed to load OpenAI engine: {}", e)
            })?;
            return Ok(LLMEngine {
                backend: EngineBackend::OpenAI(engine),
            });
        }

        #[cfg(feature = "local")]
        if let Some(llama_cfg) = cfg.llama {
            let engine = LlamaEngine::load(llama_cfg).map_err(|e| {
                error!("Failed to load Llama engine: {}", e);
                anyhow::anyhow!("Failed to load Llama engine: {}", e)
            })?;
            return Ok(LLMEngine {
                backend: EngineBackend::Llama(engine),
            });
        }

        error!("No valid LLM engine configuration found: {:?}", cfg);
        Err(anyhow!(
            "No valid LLM engine configuration found! Please check your settings."
        ))
    }

    pub async fn chat(&mut self, prompt: &str) -> Result<String> {
        match &mut self.backend {
            #[cfg(feature = "local")]
            EngineBackend::Llama(engine) => {
                let prompt_owned = prompt.to_string();

                let output_buffer = RefCell::new(String::new());
                let res = tokio::task::block_in_place(|| {
                    engine.generate_response(&prompt_owned, |token| {
                        output_buffer.borrow_mut().push_str(&token);
                    })
                });

                if let Err(e) = res {
                    error!("Llama model generation error: {}", e);
                    return Err(anyhow!("Llama error: {}", e));
                }
                Ok(output_buffer.into_inner())
            }

            #[cfg(feature = "cloud")]
            EngineBackend::OpenAI(engine) => {
                let res = engine.generate_response_sync(prompt).await;
                match res {
                    Ok(text) => Ok(text),
                    Err(e) => {
                        error!("OpenAI model generation error: {}", e);
                        Err(anyhow!("OpenAI error: {}", e))
                    }
                }
            }
        }
    }

    pub async fn chat_stream(&mut self, prompt: &str, tx: UnboundedSender<String>) {
        match &mut self.backend {
            #[cfg(feature = "local")]
            EngineBackend::Llama(engine) => {
                let res = tokio::task::block_in_place(|| {
                    engine.generate_response(prompt, move |token| {
                        let _ = tx.send(token);
                    })
                });

                if let Err(e) = res {
                    error!("Llama stream generation error: {}", e);
                }
            }

            #[cfg(feature = "cloud")]
            EngineBackend::OpenAI(engine) => {
                let res = engine.generate_response_stream(prompt, tx).await;

                if let Err(e) = res {
                    error!("OpenAI stream generation error: {}", e);
                }
            }
        }
    }

    pub fn reset_context(&mut self) {
        match &mut self.backend {
            #[cfg(feature = "local")]
            EngineBackend::Llama(engine) => engine.reset_context(),
            #[cfg(feature = "cloud")]
            EngineBackend::OpenAI(engine) => engine.reset_context(),
        }
    }
}
