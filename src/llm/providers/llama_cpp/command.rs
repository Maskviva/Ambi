// src/llm/providers/llama_cpp/command.rs

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

/// Commands that the frontend (async) can send to the engine background thread.
///
/// Each variant carries the data needed to perform the operation and the
/// necessary synchronisation primitives to send the result back (where
/// applicable).
pub(crate) enum LlamaCommand {
    /// Run a non‑streaming completion request.
    Chat {
        /// The full prompt formatted by the frontend.
        prompt: String,
        images: Vec<String>, // Multimodal payload
        /// Conversation this request belongs to; the worker keeps one cache owner at a time.
        session_id: String,
        /// Set by the async side when the request is abandoned, so the worker can stop early.
        cancel: Arc<AtomicBool>,
        /// Channel to receive the final completion or an error.
        reply_tx: oneshot::Sender<crate::error::Result<String>>,
    },
    /// Run a streaming completion request.
    ChatStream {
        prompt: String,
        images: Vec<String>, // Multimodal payload
        /// Conversation this request belongs to; the worker keeps one cache owner at a time.
        session_id: String,
        /// Set by the async side when the request is abandoned, so the worker can stop early.
        cancel: Arc<AtomicBool>,
        /// Sender for individual text chunks (each is `Ok(String)`).
        /// An `Err(...)` terminal message may be pushed when an error occurs.
        chunk_tx: mpsc::Sender<crate::error::Result<String>>,
        /// Signal that the inference loop has finished (either successfully, after
        /// an error, or because the receiver dropped).
        done_tx: oneshot::Sender<()>,
    },
    /// Clear the KV cache and all accumulated history – return to a pristine
    /// state.
    Reset,
    /// Compute the average per‑token entropy of the supplied string.
    EvaluateEntropy {
        sentence: String,
        reply_tx: oneshot::Sender<crate::error::Result<f32>>,
    },
    /// Count tokens with the model's own vocabulary, for callers that need exact numbers.
    CountTokens {
        text: String,
        reply_tx: oneshot::Sender<crate::error::Result<usize>>,
    },
    /// Politely ask the background thread to stop processing, join, and
    /// release all resources.
    Shutdown,
}
