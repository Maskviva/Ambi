// src/llm/providers/llama_cpp/dispatch.rs

use super::command::LlamaCommand;
use super::engine::LlamaEngine;
use crate::error::{AmbiError, Result};
use log::error;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

impl LlamaEngine {
    /// Send a chat completion request and wait for the full response.
    ///
    /// The caller must ensure `prompt` is already formatted.
    pub(crate) async fn chat_internal(&self, prompt: &str, images: Vec<String>) -> Result<String> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.cmd_tx
            .send(LlamaCommand::Chat {
                prompt: prompt.to_owned(),
                images,
                reply_tx,
            })
            .map_err(|_| {
                AmbiError::EngineError(
                    "Llama engine thread has terminated unexpectedly. \
                 The agent state is now invalid; please recreate the Agent instance."
                        .to_string(),
                )
            })?;

        reply_rx.await.map_err(|_| {
            AmbiError::EngineError("Reply channel closed before response".to_string())
        })?
    }

    /// Start a streaming completion and wait until the stream is exhausted.
    ///
    /// The `tx` channel is forwarded to the engine thread; incoming chunks are
    /// forwarded verbatim.  An error is logged if the thread has died.
    pub(crate) async fn stream_internal(
        &self,
        prompt: &str,
        images: Vec<String>,
        tx: Sender<Result<String>>,
    ) {
        let (done_tx, done_rx) = oneshot::channel();

        if self
            .cmd_tx
            .send(LlamaCommand::ChatStream {
                prompt: prompt.to_owned(),
                images,
                chunk_tx: tx,
                done_tx,
            })
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
        let _ = self.cmd_tx.send(LlamaCommand::Reset);
    }

    /// Evaluate the entropy of `sentence` and return the average per‑token entropy.
    pub(crate) async fn evaluate_sentence_entropy_internal(&self, sentence: &str) -> Result<f32> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.cmd_tx
            .send(LlamaCommand::EvaluateEntropy {
                sentence: sentence.to_owned(),
                reply_tx,
            })
            .map_err(|_| {
                AmbiError::EngineError("Llama engine thread terminated unexpectedly".to_string())
            })?;

        reply_rx.await.map_err(|_| {
            AmbiError::EngineError("Reply channel closed before response".to_string())
        })?
    }

    /// Exactly compute token count using the native model.
    pub(crate) async fn count_tokens_internal(&self, text: &str) -> Result<usize> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.cmd_tx
            .send(LlamaCommand::CountTokens {
                text: text.to_owned(),
                reply_tx,
            })
            .map_err(|_| {
                AmbiError::EngineError("Llama engine thread terminated unexpectedly".to_string())
            })?;

        reply_rx.await.map_err(|_| {
            AmbiError::EngineError("Reply channel closed before response".to_string())
        })?
    }
}
