// src/agent/pipeline/chat_runner/stream_handler.rs
use super::ChatRunner;
use crate::agent::tool::{StreamFormatter, ToolCallParser};
use crate::error::{AmbiError, Result};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

impl ChatRunner {
    pub(crate) async fn process_llm_stream(
        mut rx_llm: Receiver<Result<String>>,
        tx_out: &Sender<Result<String>>,
        parser: &Arc<dyn ToolCallParser>,
        enable_formatting: bool,
    ) -> (String, bool) {
        let mut full_output = String::with_capacity(1024);
        let mut formatter: Box<dyn StreamFormatter> = if enable_formatting {
            parser.create_stream_formatter()
        } else {
            Box::new(crate::agent::core::formatter::PassThroughFormatter)
        };
        let mut has_error = false;

        while let Some(result) = rx_llm.recv().await {
            match result {
                Ok(token) => {
                    full_output.push_str(&token);
                    let cleaned_text = formatter.push(&token);

                    if !cleaned_text.is_empty() && tx_out.send(Ok(cleaned_text)).await.is_err() {
                        log::warn!("Client disconnected, aborting LLM stream");
                        has_error = true;
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx_out
                        .send(Err(AmbiError::EngineError(format!(
                            "LLM Engine Error: {}",
                            e
                        ))))
                        .await;
                    has_error = true;
                    break;
                }
            }
        }

        if !has_error {
            let flushed = formatter.flush();
            if !flushed.is_empty() {
                let _ = tx_out.send(Ok(flushed)).await;
            }
        }
        (full_output, has_error)
    }
}
