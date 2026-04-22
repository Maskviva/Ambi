use crate::agent::core::{Agent, CompletionRequest, EvictionHandler};
use crate::agent::tool::{DynTool, StreamFormatter, ToolCallParser, ToolManager};
use crate::types::message::Message;

use anyhow::{anyhow, Result};
use futures::stream::{self, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::wrappers::ReceiverStream;

#[async_trait::async_trait]
pub trait ChatPipeline {
    async fn chat(&mut self, prompt: &str) -> Result<String>;
    async fn chat_stream(
        &mut self,
        prompt: &str,
    ) -> Result<Pin<Box<ReceiverStream<Result<String, String>>>>, ()>;
    async fn clear_history(&self);
}

#[async_trait::async_trait]
impl ChatPipeline for Agent {
    async fn chat(&mut self, prompt: &str) -> Result<String> {
        let mut engine = self
            .llm_engine
            .try_lock()
            .map_err(|_| anyhow!("Agent is currently busy processing another request."))?;

        Self::append_user_message(&self.completion_request, prompt).await;
        let mut snapshot_len = self.completion_request.lock().await.chat_history.len();

        let mut final_formatted_output = String::new();
        let mut iteration_count = 0;

        loop {
            if iteration_count >= self.max_iterations {
                self.completion_request
                    .lock()
                    .await
                    .chat_history
                    .truncate(snapshot_len);
                return Err(anyhow!(
                    "Agent has reached the maximum number of tool call loops."
                ));
            }

            let req_data = Self::get_llm_request(
                &self.completion_request,
                &self.system_prompt,
                &self.template,
                &self.cached_tool_prompt,
            )
            .await;

            let res = match engine.chat(req_data).await {
                Ok(r) => r,
                Err(e) => {
                    self.completion_request
                        .lock()
                        .await
                        .chat_history
                        .truncate(snapshot_len);
                    return Err(e);
                }
            };

            let mut dynamic_system_overhead = 0;
            for msg in self.completion_request.lock().await.chat_history.all() {
                if matches!(**msg, Message::System { .. }) {
                    dynamic_system_overhead += msg.text_len() / 4;
                }
            }
            let prompt_overhead = (self.system_prompt.len() + self.cached_tool_prompt.len()) / 4
                + dynamic_system_overhead;

            let evicted_count = Self::append_assistant_message_and_evict(
                &self.completion_request,
                res.clone(),
                &self.on_evict_handler,
                self.eviction_strategy,
                prompt_overhead,
            )
            .await;
            snapshot_len = snapshot_len.saturating_sub(evicted_count);

            let mut formatter: Box<dyn StreamFormatter> = if self.enable_formatting {
                self.tool_parser.create_stream_formatter()
            } else {
                Box::new(crate::agent::core::formatter::PassThroughFormatter)
            };

            final_formatted_output.push_str(&formatter.push(&res));
            final_formatted_output.push_str(&formatter.flush());

            let tool_calls = match Self::handle_tool_calls(
                &self.completion_request,
                Arc::clone(&self.tool_map),
                &self.tool_parser,
                &res,
                None,
            )
            .await
            {
                Ok(calls) => calls,
                Err(e) => {
                    self.completion_request
                        .lock()
                        .await
                        .chat_history
                        .truncate(snapshot_len);
                    return Err(e);
                }
            };

            if tool_calls.is_empty() {
                return Ok(final_formatted_output.trim().to_string());
            }

            Self::process_tool_calls_output(&tool_calls, &mut final_formatted_output);
            iteration_count += 1;
        }
    }

    async fn chat_stream(
        &mut self,
        prompt: &str,
    ) -> Result<Pin<Box<ReceiverStream<Result<String, String>>>>, ()> {
        let mut owned_engine = match Arc::clone(&self.llm_engine).try_lock_owned() {
            Ok(guard) => guard,
            Err(_) => return Err(()),
        };

        let completion_request = Arc::clone(&self.completion_request);
        let system_prompt = self.system_prompt.clone();
        let prompt_clone = prompt.to_string();

        let (tx_out, rx_out) = channel::<Result<String, String>>(1024);

        let template_clone = self.template.clone();
        let tool_map_clone = Arc::clone(&self.tool_map);
        let tool_parser_clone = Arc::clone(&self.tool_parser);
        let evict_handler_clone = self.on_evict_handler.clone();
        let max_iterations = self.max_iterations;
        let enable_formatting = self.enable_formatting;
        let eviction_strategy = self.eviction_strategy;
        let cached_tool_prompt = self.cached_tool_prompt.clone();

        tokio::spawn(async move {
            Self::append_user_message(&completion_request, &prompt_clone).await;
            let mut snapshot_len = completion_request.lock().await.chat_history.len();
            let mut iteration_count = 0;

            loop {
                if iteration_count >= max_iterations {
                    let _ = tx_out.send(Err("Max loops reached.".to_string())).await;
                    completion_request
                        .lock()
                        .await
                        .chat_history
                        .truncate(snapshot_len);
                    break;
                }

                let req_data = Self::get_llm_request(
                    &completion_request,
                    &system_prompt,
                    &template_clone,
                    &cached_tool_prompt,
                )
                .await;

                let (tx_llm, rx_llm) = channel::<Result<String, anyhow::Error>>(1024);

                let process_future = Self::process_llm_stream(
                    rx_llm,
                    &tx_out,
                    &tool_parser_clone,
                    enable_formatting,
                );

                let engine_future = async { owned_engine.chat_stream(req_data, tx_llm).await };

                let (_, (full_output, has_error)) = tokio::join!(engine_future, process_future);

                if has_error {
                    completion_request
                        .lock()
                        .await
                        .chat_history
                        .truncate(snapshot_len);
                    break;
                }

                let mut dynamic_system_overhead = 0;
                for msg in completion_request.lock().await.chat_history.all() {
                    if matches!(**msg, Message::System { .. }) {
                        dynamic_system_overhead += msg.text_len() / 4;
                    }
                }

                let prompt_overhead =
                    (system_prompt.len() + cached_tool_prompt.len()) / 4 + dynamic_system_overhead;

                let evicted_count = Self::append_assistant_message_and_evict(
                    &completion_request,
                    full_output.clone(),
                    &evict_handler_clone,
                    eviction_strategy,
                    prompt_overhead,
                )
                .await;
                snapshot_len = snapshot_len.saturating_sub(evicted_count);

                let tool_calls = match Self::handle_tool_calls(
                    &completion_request,
                    Arc::clone(&tool_map_clone),
                    &tool_parser_clone,
                    &full_output,
                    Some(tx_out.clone()),
                )
                .await
                {
                    Ok(calls) => calls,
                    Err(e) => {
                        let _ = tx_out.send(Err(format!("Tool call error: {}", e))).await;
                        completion_request
                            .lock()
                            .await
                            .chat_history
                            .truncate(snapshot_len);
                        break;
                    }
                };

                if tool_calls.is_empty() {
                    break;
                }

                let mut formatted_tools = String::new();
                Self::process_tool_calls_output(&tool_calls, &mut formatted_tools);
                let _ = tx_out.send(Ok(formatted_tools)).await;

                iteration_count += 1;
            }
        });

        Ok(Box::pin(ReceiverStream::new(rx_out)))
    }

    async fn clear_history(&self) {
        self.completion_request.lock().await.chat_history.clear();
        self.llm_engine.lock().await.reset_context();
    }
}

impl Agent {
    async fn append_user_message(req_mutex: &TokioMutex<CompletionRequest>, prompt: &str) {
        req_mutex
            .lock()
            .await
            .chat_history
            .push(Message::user_text(prompt));
    }

    async fn append_assistant_message_and_evict(
        req_mutex: &TokioMutex<CompletionRequest>,
        content: String,
        handler: &Option<EvictionHandler>,
        eviction_strategy: (usize, usize, usize),
        prompt_overhead: usize,
    ) -> usize {
        let evicted_msgs = {
            let mut req = req_mutex.lock().await;
            req.chat_history.push(Message::Assistant { content });
            req.chat_history.evict_old_messages(
                eviction_strategy.0,
                eviction_strategy.1,
                eviction_strategy.2,
                prompt_overhead,
            )
        };
        let count = evicted_msgs.len();
        if count > 0 {
            log::debug!("Context truncation: Evicted {} messages.", count);
            if let Some(h) = handler {
                h(evicted_msgs);
            }
        }
        count
    }

    async fn process_llm_stream(
        mut rx_llm: Receiver<Result<String, anyhow::Error>>,
        tx_out: &Sender<Result<String, String>>,
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
                    let _ = tx_out.send(Err(format!("LLM Engine Error: {}", e))).await;
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

    fn process_tool_calls_output(
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

    async fn handle_tool_calls(
        req_mutex: &TokioMutex<CompletionRequest>,
        tool_map: Arc<HashMap<String, Arc<dyn DynTool>>>,
        parser: &Arc<dyn ToolCallParser>,
        assistant_response: &str,
        tx_out: Option<Sender<Result<String, String>>>,
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
                return Err(anyhow!("Client disconnected during tool execution"));
            }

            req_mutex.lock().await.chat_history.push(Message::Tool {
                content: msg.clone(),
            });
            results.push((name, args_str, msg));
        }

        Ok(results)
    }
}
