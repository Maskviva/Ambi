// src/llm/providers/llama_cpp/engine/engine.rs

use crate::error::Result;
use crate::llm::providers::llama_cpp::command::LlamaCommand;
use crate::llm::providers::llama_cpp::thread;
use crate::llm::LLMEngineTrait;
use crate::types::config::LlamaEngineConfig;
use crate::types::LLMRequest;
use async_trait::async_trait;
use std::thread::JoinHandle;
use tokio::sync::mpsc::{Sender, UnboundedSender};

/// The public handle to the llama.cpp inference engine.
///
/// All operations are non‑blocking on the async runtime.  A background thread
/// owns the heavy model resources and serialises inference requests.
pub struct LlamaEngine {
    /// Sends commands to the background thread.
    pub(crate) cmd_tx: UnboundedSender<LlamaCommand>,
    /// Join handle for the background thread (used during graceful shutdown).
    _handle: Option<JoinHandle<()>>,
}

impl LlamaEngine {
    /// Create and start the engine.
    ///
    /// # Errors
    ///
    /// Returns `AmbiError::EngineError` if the background thread could not be
    /// spawned (extremely unlikely) or if the configuration fails validation.
    pub fn load(cfg: LlamaEngineConfig) -> Result<Self> {
        cfg.validate()?;

        unsafe {
            llama_cpp_sys_2::llama_log_set(
                Some(super::callback::llama_log_callback),
                std::ptr::null_mut(),
            );
        }

        let (cmd_tx, handle) = thread::spawn_engine_thread(cfg)?;

        Ok(Self {
            cmd_tx,
            _handle: Some(handle),
        })
    }
}

impl Drop for LlamaEngine {
    fn drop(&mut self) {
        // Attempt polite shutdown – ignore errors as the thread may already be dead.
        let _ = self.cmd_tx.send(LlamaCommand::Shutdown);
        // Wait for the background thread to finish to ensure resources are freed.
        if let Some(handle) = self._handle.take() {
            let _ = handle.join();
        }
    }
}

#[async_trait]
impl LLMEngineTrait for LlamaEngine {
    async fn chat(&mut self, request: LLMRequest) -> Result<String> {
        self.chat_internal(&request.formatted_prompt).await
    }

    async fn chat_stream(&mut self, request: LLMRequest, tx: Sender<Result<String>>) {
        self.stream_internal(&request.formatted_prompt, tx).await;
    }

    fn reset_context(&mut self) {
        self.reset_internal();
    }

    async fn evaluate_sentence_entropy(&mut self, sentence: &str) -> Result<f32> {
        self.evaluate_sentence_entropy_internal(sentence).await
    }
}
