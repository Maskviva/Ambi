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
    ) -> (String, Option<AmbiError>) {
        let mut full_output = String::with_capacity(1024);
        let mut formatter: Box<dyn StreamFormatter> = if enable_formatting {
            parser.create_stream_formatter()
        } else {
            Box::new(crate::agent::core::formatter::PassThroughFormatter)
        };
        let mut last_error: Option<AmbiError> = None;

        while let Some(result) = rx_llm.recv().await {
            match result {
                Ok(token) => {
                    full_output.push_str(&token);
                    let cleaned_text = formatter.push(&token);
                    if !cleaned_text.is_empty() && tx_out.send(Ok(cleaned_text)).await.is_err() {
                        log::warn!("Client disconnected, aborting LLM stream");
                        last_error = Some(AmbiError::PipelineError(
                            "Client disconnected during stream".into(),
                        ));
                        break;
                    }
                }
                Err(e) => {
                    let engine_err = AmbiError::EngineError(format!("LLM Engine Error: {}", e));
                    let _ = tx_out.send(Err(engine_err.clone())).await;
                    last_error = Some(engine_err);
                    break;
                }
            }
        }

        // 流结束，flush 剩余缓冲
        if last_error.is_none() {
            let flushed = formatter.flush();
            if !flushed.is_empty() {
                let _ = tx_out.send(Ok(flushed)).await;
            }
        }

        (full_output, last_error)
    }
}
