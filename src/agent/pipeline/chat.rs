use crate::agent::core::formatter::StreamFormatter;
use crate::agent::core::{Agent, CompletionRequest};
use crate::agent::tool::{DynTool, ToolCallParser, ToolManager};
use crate::types::message::Message;

use anyhow::{anyhow, Result};
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
        Self::append_user_message(&self.completion_request, prompt).await;

        let mut target = prompt.to_string();
        let mut final_formatted_output = String::new();
        let mut iteration_count = 0;

        loop {
            if iteration_count >= self.max_iterations {
                return Err(anyhow!("Agent has reached the maximum number of tool call loops ({}), forcibly terminating.", self.max_iterations));
            }

            let req_data = Self::get_llm_request(
                &self.completion_request,
                &self.system_prompt,
                &self.template,
                &self.tools_def,
            )
            .await;

            let res = self.llm_engine.lock().await.chat(req_data).await?;

            Self::append_assistant_message_and_evict(
                &self.completion_request,
                target.clone(),
                res.clone(),
                &self.on_evict_handler,
            )
            .await;

            let mut formatter = StreamFormatter::new();
            final_formatted_output.push_str(&formatter.push(&res));
            final_formatted_output.push_str(&formatter.flush());

            let tool_calls = Self::handle_tool_calls(
                &self.completion_request,
                &self.tool_map,
                &self.tool_parser,
                &res,
            )
            .await?;

            if tool_calls.is_empty() {
                return Ok(final_formatted_output.trim().to_string());
            }

            target = Self::process_tool_calls_output(&tool_calls, &mut final_formatted_output);
            iteration_count += 1;
        }
    }

    async fn chat_stream(
        &mut self,
        prompt: &str,
    ) -> Result<Pin<Box<ReceiverStream<Result<String, String>>>>, ()> {
        let llm_engine = Arc::clone(&self.llm_engine);
        let completion_request = Arc::clone(&self.completion_request);
        let system_prompt = self.system_prompt.clone();

        Self::append_user_message(&completion_request, prompt).await;

        let prompt_clone = prompt.to_string();
        let (tx_out, rx_out) = channel::<Result<String, String>>(1024);

        let template_clone = self.template.clone();
        let tools_def_clone = Arc::clone(&self.tools_def);
        let tool_map_clone = Arc::clone(&self.tool_map);
        let tool_parser_clone = Arc::clone(&self.tool_parser);
        let evict_handler_clone = self.on_evict_handler.clone();
        let max_iterations = self.max_iterations;

        tokio::spawn(async move {
            let mut target = prompt_clone.clone();
            let mut iteration_count = 0;

            loop {
                if iteration_count >= max_iterations {
                    let _ = tx_out
                        .send(Err(format!(
                            "Agent has reached the maximum number of tool call loops ({}), forcibly terminating.",
                            max_iterations
                        )))
                        .await;
                    break;
                }

                let req_data = Self::get_llm_request(
                    &completion_request,
                    &system_prompt,
                    &template_clone,
                    &tools_def_clone,
                )
                .await;

                let (tx_llm, mut rx_llm) = channel::<Result<String, anyhow::Error>>(1024);
                let llm_engine_clone = Arc::clone(&llm_engine);

                let llm_task = tokio::spawn(async move {
                    let mut engine = llm_engine_clone.lock().await;
                    engine.chat_stream(req_data, tx_llm).await;
                });

                let (full_output, has_error) = Self::process_llm_stream(&mut rx_llm, &tx_out).await;

                if has_error {
                    break;
                }
                let _ = llm_task.await;

                Self::append_assistant_message_and_evict(
                    &completion_request,
                    target.clone(),
                    full_output.clone(),
                    &evict_handler_clone,
                )
                .await;

                let tool_calls = match Self::handle_tool_calls(
                    &completion_request,
                    &tool_map_clone,
                    &tool_parser_clone,
                    &full_output,
                )
                .await
                {
                    Ok(calls) => calls,
                    Err(e) => {
                        let _ = tx_out.send(Err(format!("Tool call error: {}", e))).await;
                        break;
                    }
                };

                if tool_calls.is_empty() {
                    break;
                }

                let mut formatted_tools = String::new();
                target = Self::process_tool_calls_output(&tool_calls, &mut formatted_tools);
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
        target: String,
        content: String,
        handler: &Option<Arc<dyn Fn(Vec<Message>) + Send + Sync>>,
    ) {
        let evicted_msgs = {
            let mut req = req_mutex.lock().await;
            req.chat_history
                .push(Message::Assistant { target, content });
            req.__requested = true;
            req.chat_history.evict_old_messages(2, 6)
        };

        if !evicted_msgs.is_empty() {
            log::debug!(
                "Context truncation: Evicted {} messages.",
                evicted_msgs.len()
            );
            if let Some(h) = handler {
                h(evicted_msgs);
            }
        }
    }

    async fn process_llm_stream(
        rx_llm: &mut Receiver<Result<String, anyhow::Error>>,
        tx_out: &Sender<Result<String, String>>,
    ) -> (String, bool) {
        let mut full_output = String::with_capacity(1024);
        let mut formatter = StreamFormatter::new();
        let mut has_error = false;

        while let Some(result) = rx_llm.recv().await {
            match result {
                Ok(token) => {
                    full_output.push_str(&token);
                    let cleaned_text = formatter.push(&token);
                    if !cleaned_text.is_empty() {
                        let _ = tx_out.send(Ok(cleaned_text)).await;
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
    ) -> String {
        let mut last_target = String::new();
        for (name, args, tool_msg) in tool_calls {
            let formatted_tool_block = format!(
                "\n\n[TOOL_CALL]: {}({})\n[TOOL]: {}\n\n",
                name, args, tool_msg
            );
            output_buffer.push_str(&formatted_tool_block);
            last_target = tool_msg.clone();
        }
        last_target
    }

    async fn handle_tool_calls(
        req_mutex: &TokioMutex<CompletionRequest>,
        tool_map: &HashMap<String, Arc<dyn DynTool>>,
        parser: &Arc<dyn ToolCallParser>,
        assistant_response: &str,
    ) -> Result<Vec<(String, String, String)>> {
        let calls = parser.parse(assistant_response);
        let mut results = Vec::new();

        for (name, args) in calls {
            let tool_result = ToolManager::run_tool(tool_map, name.clone(), &args).await;

            let tool_msg =
                tool_result.unwrap_or_else(|e| format!("Failed to execute tool '{}': {}", name, e));

            req_mutex.lock().await.chat_history.push(Message::Tool {
                target: assistant_response.to_string(),
                content: tool_msg.clone(),
            });

            results.push((name, args.to_string(), tool_msg));
        }

        Ok(results)
    }
}
