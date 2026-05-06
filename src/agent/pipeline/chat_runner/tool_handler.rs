// src/agent/pipeline/chat_runner/tool_handler.rs

use super::{ChatRunner, StateManager};
use crate::agent::core::DynToolObj; // <-- Import Type Alias
use crate::agent::tool::ToolManager;
use crate::error::{AmbiError, Result};
use crate::types::Message;

use futures::stream::{self, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

impl ChatRunner {
    /// To present generated pseudo-labels to humans (injecting into the output stream) and inform about tool usage.
    /// This string is only for UI presentation and will not flow into the Agent's actual context.
    pub(crate) fn process_tool_calls_output(
        tool_calls: &[(String, String, String)],
        output_buffer: &mut String,
    ) {
        for (name, args, _tool_msg) in tool_calls {
            if name == "__format_error__" {
                let err_msg = "\n\n[SYSTEM: Tool call format error - your previous JSON was invalid. Please correct it.]\n\n".to_string();
                output_buffer.push_str(&err_msg);
                continue;
            }
            let formatted_tool_block = format!("\n\n[TOOL_CALL]: {}({})\n\n", name, args);
            output_buffer.push_str(&formatted_tool_block);
        }
    }

    /// Execute the parsed tools with high concurrency and implement a ghost blocking mechanism within them.
    pub(crate) async fn handle_tool_calls(
        state_accessor: &StateManager<'_>,
        engine: &crate::llm::LLMEngine,
        maximum_concurrency: usize,
        tool_map: Arc<HashMap<String, Arc<DynToolObj>>>,
        calls: Vec<(String, serde_json::Value, String)>,
        tx_out: Option<Sender<Result<String>>>,
    ) -> Result<Vec<(String, String, String)>> {
        let mut results = Vec::new();

        // Schedule multiple tools concurrently (maximum concurrency 5)
        let mut stream = stream::iter(calls)
            .map(move |(name, args, id)| {
                let t_map = Arc::clone(&tool_map);
                let tx_clone = tx_out.clone();
                async move {
                    if name == "__format_error__" {
                        let raw = args.get("raw").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let err_json = serde_json::json!({
                            "status": "error",
                            "error_type": "invalid_json_format",
                            "message": "The tool arguments provided are not valid JSON.",
                            "raw_input": raw,
                            "suggestion": "Please ensure your output strictly follows valid JSON syntax without trailing commas or unescaped quotes."
                        });

                        return Ok((name, args.to_string(), err_json.to_string(), id));
                    }

                    let run_future = ToolManager::run_tool(&t_map, name.clone(), &args);

                    tokio::select! {
                        res = run_future => {
                            let msg = res.unwrap_or_else(|e| {
                                serde_json::json!({
                                    "status": "error",
                                    "error_type": "execution_failed",
                                    "message": e.to_string()
                                }).to_string()
                            });
                            Ok((name, args.to_string(), msg, id))
                        }

                       // Ghost call cancellation mechanism: monitor whether the external communication channel is disconnected
                        // Once the user actively disconnects (the stream channel is closed), immediately discard long-running background operations such as web crawlers and databases.
                         _ = async {
                            if let Some(tx) = tx_clone {
                                tx.closed().await;
                            } else {
                                std::future::pending::<()>().await;
                            }
                        } => {
                            log::error!("Client disconnected. Aborting ghost tool execution: {}", name);
                            Err(AmbiError::AgentError(
                                "Client disconnected during tool execution".to_string(),
                            ))
                        }
                    }
                }
            })
            .buffered(maximum_concurrency);

        // Collect the results of all concurrent tools and record them in the history database separately
        while let Some(res) = stream.next().await {
            let (name, args_str, msg, id) = res?;

            let tool_msg = Message::Tool {
                content: msg.clone(),
                tool_id: Some(id.clone()),
            };

            let tokens = engine.count_tokens(&tool_msg.to_string())?;

            state_accessor
                .push_tool_message(msg.clone(), Some(id), tokens)
                .await?;
            results.push((name, args_str, msg));
        }

        Ok(results)
    }
}
