// src/llm/providers/llama_cpp/dispatch.rs

use super::command::LlamaCommand;
use super::engine::LlamaEngine;
use crate::error::{AmbiError, Result};
use log::error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

/// Flips the shared cancellation flag when the awaiting side goes away.
///
/// The inference worker is a plain OS thread, so dropping the future on the async side has no
/// effect on it by itself. Holding this guard inside the awaiting function ties the two together:
/// if the future is dropped (timeout, client disconnect, cancelled task), the flag is set and the
/// worker stops at its next token instead of decoding to `max_tokens`.
struct CancelOnDrop(Arc<AtomicBool>);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

impl LlamaEngine {
    /// Send a chat completion request and wait for the full response.
    ///
    /// The caller must ensure `prompt` is already formatted.
    pub(crate) async fn chat_internal(
        &self,
        prompt: &str,
        images: Vec<String>,
        session_id: &str,
    ) -> Result<String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let _guard = CancelOnDrop(Arc::clone(&cancel));

        self.cmd_tx
            .send(LlamaCommand::Chat {
                prompt: prompt.to_owned(),
                images,
                session_id: session_id.to_owned(),
                cancel,
                reply_tx,
            })
            .await
            .map_err(|_| {
                AmbiError::engine(
                    "Llama engine thread has terminated unexpectedly. \
                 The agent state is now invalid; please recreate the Agent instance."
                        .to_string(),
                )
            })?;

        reply_rx
            .await
            .map_err(|_| AmbiError::engine("Reply channel closed before response".to_string()))?
    }

    /// Start a streaming completion and wait until the stream is exhausted.
    ///
    /// The `tx` channel is forwarded to the engine thread; incoming chunks are
    /// forwarded verbatim. An error is logged if the thread has died.
    pub(crate) async fn stream_internal(
        &self,
        prompt: &str,
        images: Vec<String>,
        session_id: &str,
        tx: Sender<Result<String>>,
    ) {
        let (done_tx, done_rx) = oneshot::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let _guard = CancelOnDrop(Arc::clone(&cancel));

        if self
            .cmd_tx
            .send(LlamaCommand::ChatStream {
                prompt: prompt.to_owned(),
                images,
                session_id: session_id.to_owned(),
                cancel,
                chunk_tx: tx,
                done_tx,
            })
            .await
            .is_err()
        {
            error!("Llama engine thread terminated unexpectedly – cannot start stream");
            return;
        }

        let _ = done_rx.await;
    }

    /// Request a full KV‑cache and history reset (fire‑and‑forget).
    ///
    /// Even if the thread has died the error is silently swallowed – the next
    /// chat call will surface the problem.
    pub(crate) fn reset_internal(&self) {
        let _ = self.cmd_tx.try_send(LlamaCommand::Reset);
    }

    /// Ask the worker to release everything. Used from `Drop`, so it must never block.
    pub(crate) fn shutdown_internal(&self) {
        let _ = self.cmd_tx.try_send(LlamaCommand::Shutdown);
    }

    /// Evaluate the entropy of `sentence` and return the average per‑token entropy.
    pub(crate) async fn evaluate_sentence_entropy_internal(&self, sentence: &str) -> Result<f32> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.cmd_tx
            .send(LlamaCommand::EvaluateEntropy {
                sentence: sentence.to_owned(),
                reply_tx,
            })
            .await
            .map_err(|_| {
                AmbiError::engine("Llama engine thread terminated unexpectedly".to_string())
            })?;

        reply_rx
            .await
            .map_err(|_| AmbiError::engine("Reply channel closed before response".to_string()))?
    }

    /// Exactly compute token count using the model's own vocabulary.
    ///
    /// The framework's eviction accounting uses the fast, approximate
    /// [`crate::llm::tokenizer::DefaultTokenizer`]; this method is the escape hatch for callers
    /// that need the real number and can afford a round trip to the worker.
    pub async fn count_tokens_exact(&self, text: &str) -> Result<usize> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.cmd_tx
            .send(LlamaCommand::CountTokens {
                text: text.to_owned(),
                reply_tx,
            })
            .await
            .map_err(|_| {
                AmbiError::engine("Llama engine thread terminated unexpectedly".to_string())
            })?;

        reply_rx
            .await
            .map_err(|_| AmbiError::engine("Reply channel closed before response".to_string()))?
    }
}
