#[cfg(feature = "local")]
use crate::core::llm::engine::LlamaEngine;

#[cfg(feature = "cloud")]
use crate::core::llm::openai::OpenAIEngine;

use crate::core::llm::{EngineBackend, EngineConfig};
use crate::utils::LLM_LOGGER;
use anyhow::{anyhow, Result};
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
                LLM_LOGGER.errorf(format_args!("OpenAI引擎加载失败: {}", e));
                anyhow::anyhow!("OpenAI引擎加载失败: {}", e)
            })?;
            return Ok(LLMEngine {
                backend: EngineBackend::OpenAI(engine),
            });
        }

        #[cfg(feature = "local")]
        if let Some(llama_cfg) = cfg.llama {
            let engine = LlamaEngine::load(llama_cfg).map_err(|e| {
                LLM_LOGGER.errorf(format_args!("Llama引擎加载失败: {}", e));
                anyhow::anyhow!("Llama引擎加载失败: {}", e)
            })?;
            return Ok(LLMEngine {
                backend: EngineBackend::Llama(engine),
            });
        }

        LLM_LOGGER.errorf(format_args!("未找到任何有效的 LLM 引擎配置 {:?}", cfg));
        panic!("未找到任何有效的 LLM 引擎配置！请检查配置文件。");
    }

    pub async fn chat(&mut self, prompt: &str, is_tool_call: bool) -> Result<String> {
        if !is_tool_call {
            self.reset_context();
        }

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
                    LLM_LOGGER.errorf(format_args!("Llama 模型输出出错: {}", e));
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
                        LLM_LOGGER.errorf(format_args!("OpenAI 模型输出出错: {}", e));
                        Err(anyhow!("OpenAI error: {}", e))
                    }
                }
            }
        }
    }

    pub async fn chat_stream(
        &mut self,
        prompt: &str,
        is_tool_call: bool,
        tx: UnboundedSender<String>,
    ) {
        if !is_tool_call {
            self.reset_context();
        }

        match &mut self.backend {
            #[cfg(feature = "local")]
            EngineBackend::Llama(engine) => {
                let res = tokio::task::block_in_place(|| {
                    engine.generate_response(prompt, move |token| {
                        let _ = tx.send(token);
                    })
                });

                if let Err(e) = res {
                    LLM_LOGGER.errorf(format_args!("Llama Stream输出出错: {}", e));
                }
            }

            #[cfg(feature = "cloud")]
            EngineBackend::OpenAI(engine) => {
                let res = engine.generate_response_stream(prompt, tx).await;

                if let Err(e) = res {
                    LLM_LOGGER.errorf(format_args!("OpenAI Stream输出出错: {}", e));
                }
            }
        }
    }

    fn reset_context(&mut self) {
        match &mut self.backend {
            #[cfg(feature = "local")]
            EngineBackend::Llama(engine) => engine.reset_context(),
            #[cfg(feature = "cloud")]
            EngineBackend::OpenAI(engine) => engine.reset_context(),
        }
    }
}
