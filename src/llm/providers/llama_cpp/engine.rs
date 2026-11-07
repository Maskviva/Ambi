// src/llm/providers/llama_cpp/engine.rs

use super::command::LlamaCommand;
use super::config::LlamaEngineConfig;
use super::thread;
use crate::error::Result;
use crate::impl_as_any;
use crate::llm::LLMEngineTrait;
use crate::types::LLMRequest;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use tokio::sync::mpsc::Sender;

/// The public handle to the llama.cpp inference engine.
///
/// All operations are non‑blocking on the async runtime. A background thread owns the heavy model
/// resources and serialises inference requests.
///
/// # Concurrency
///
/// The worker serves one request at a time. The command queue is bounded, so submitting beyond
/// that depth applies backpressure to the caller instead of growing memory without limit.
///
/// # Session state
///
/// Each request carries the identity of the conversation it belongs to. Because a context holds a
/// single sequence, switching conversations begins from a clean cache; alternating conversations
/// on one engine therefore pays a re-prefill cost rather than silently sharing decoder state.
pub struct LlamaEngine {
    /// Sends commands to the background thread.
    pub(crate) cmd_tx: Sender<LlamaCommand>,
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

    /// Evaluates the entropy of a sentence.
    ///
    /// The sentence is decoded in a throwaway context, so this measurement never disturbs the
    /// chat sequence.
    pub async fn evaluate_sentence_entropy(&self, sentence: &str) -> Result<f32> {
        self.evaluate_sentence_entropy_internal(sentence).await
    }
}

impl Drop for LlamaEngine {
    fn drop(&mut self) {
        // Polite shutdown; never block in `Drop`, so a full queue simply skips the courtesy call.
        self.shutdown_internal();

        self._handle.take();
    }
}

#[async_trait]
impl LLMEngineTrait for LlamaEngine {
    impl_as_any!();

    async fn chat(&self, request: LLMRequest) -> Result<String> {
        self.chat_internal(
            &request.formatted_prompt,
            request.images,
            &request.session_id,
        )
        .await
    }

    async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>) {
        self.stream_internal(
            &request.formatted_prompt,
            request.images,
            &request.session_id,
            tx,
        )
        .await;
    }

    fn reset_context(&self) {
        self.reset_internal();
    }

    fn supports_multimodal(&self) -> bool {
        self._supports_multimodal
    }
}
