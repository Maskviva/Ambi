// src/llm/providers/llama_cpp/engine.rs

use super::command::LlamaCommand;
use super::config::LlamaEngineConfig;
use super::thread;
use crate::error::Result;
use crate::llm::LLMEngineTrait;
use crate::types::LLMRequest;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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

    pub(crate) _supports_multimodal: bool,

    alive: Arc<AtomicBool>,
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

        // Dynamically determine multimodal capabilities based on config flags
        let supports_multimodal = cfg.mmproj_path.is_some() || cfg.integrated_vision;

        unsafe {
            llama_cpp_sys_2::llama_log_set(
                Some(super::callback::llama_log_callback),
                std::ptr::null_mut(),
            );
        }

        let (cmd_tx, handle, alive) = thread::spawn_engine_thread(cfg)?;

        Ok(Self {
            cmd_tx,
            _handle: Some(handle),
            _supports_multimodal: supports_multimodal,
            alive,
        })
    }

    /// Checks if the Llama.cpp engine process is still alive.
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
}

impl Drop for LlamaEngine {
    fn drop(&mut self) {
        // Attempt polite shutdown – ignore errors as the thread may already be dead.
        let _ = self.cmd_tx.send(LlamaCommand::Shutdown);

        self._handle.take();
    }
}

#[async_trait]
impl LLMEngineTrait for LlamaEngine {
    async fn chat(&self, request: LLMRequest) -> Result<String> {
        self.chat_internal(&request.formatted_prompt, request.images)
            .await
    }

    async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>) {
        self.stream_internal(&request.formatted_prompt, request.images, tx)
            .await;
    }

    fn reset_context(&self) {
        self.reset_internal();
    }

    fn supports_multimodal(&self) -> bool {
        self._supports_multimodal
    }

    async fn evaluate_sentence_entropy(&self, sentence: &str) -> Result<f32> {
        self.evaluate_sentence_entropy_internal(sentence).await
    }
}
