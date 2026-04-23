// src/agent/pipeline/chat_runner/tool_handler.rs
use super::ChatRunner;
use crate::agent::core::CompletionRequest;
use crate::agent::tool::{DynTool, ToolCallParser, ToolManager};
use crate::error::{AmbiError, Result};
use crate::types::message::Message;
use futures::stream::{self, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex as TokioMutex;

impl ChatRunner {
    pub(crate) fn process_tool_calls_output(
        tool_calls: &[(String, String, String)],
        output_buffer: &mut String,
    ) {
        for (name, args, _tool_msg) in tool_calls {
            if name == "__format_error__" {
                continue;
            }
            let formatted_tool_block = format!("\n\n[TOOL_CALL]: {}({})\n\n", name, args);
            output_buffer.push_str(&formatted_tool_block);
        }
    }

    pub(crate) async fn handle_tool_calls(
        req_mutex: &TokioMutex<CompletionRequest>,
        tool_map: Arc<HashMap<String, Arc<dyn DynTool>>>,
        parser: &Arc<dyn ToolCallParser>,
        assistant_response: &str,
        tx_out: Option<Sender<Result<String>>>,
    ) -> Result<Vec<(String, String, String)>> {
        let calls = parser.parse(assistant_response);
        let mut results = Vec::new();

        let mut stream = stream::iter(calls)
            .map(move |(name, args)| {
                let t_map = Arc::clone(&tool_map);
                let tx_clone = tx_out.clone();
                async move {
                    if name == "__format_error__" {
                        let raw = args.get("raw").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        return (name, args.to_string(), format!("CRITICAL ERROR: Invalid JSON format. Raw: {}", raw));
                    }

                    let run_future = ToolManager::run_tool(&t_map, name.clone(), &args);

                    tokio::select! {
                        res = run_future => {
                            let msg = res.unwrap_or_else(|e| format!("Failed to execute '{}': {}", name, e));
                            (name, args.to_string(), msg)
                        }
                        _ = async {
                            if let Some(tx) = tx_clone {
                                tx.closed().await;
                            } else {
                                std::future::pending::<()>().await;
                            }
                        } => {
                            log::error!("Client disconnected. Aborting ghost tool execution: {}", name);
                            (name, args.to_string(), "CRITICAL ERROR: Client disconnected".to_string())
                        }
                    }
                }
            })
            .buffered(5);

        while let Some((name, args_str, msg)) = stream.next().await {
            if msg.contains("CRITICAL ERROR: Client disconnected") {
                return Err(AmbiError::AgentError(
                    "Client disconnected during tool execution".to_string(),
                ));
            }

            req_mutex.lock().await.chat_history.push(Message::Tool {
                content: msg.clone(),
            });
            results.push((name, args_str, msg));
        }

        Ok(results)
    }
}
