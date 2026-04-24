use crate::agent::tool::StreamFormatter;
use crate::error::AmbiError;
use crate::{Agent, ChatRunner, ContentPart, Message};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;

impl ChatRunner {
    pub async fn chat_multimodal(
        agent: &mut Agent,
        parts: Vec<ContentPart>,
    ) -> crate::error::Result<String> {
        let mut engine = agent
            .llm_engine
            .try_lock()
            .map_err(|_| AmbiError::AgentBusy)?;

        Self::append_user_message(&agent.completion_request, parts).await;

        let mut snapshot_len = agent.completion_request.lock().await.chat_history.len();

        let mut final_formatted_output = String::new();
        let mut iteration_count = 0;

        loop {
            if iteration_count >= agent.config.max_iterations {
                agent
                    .completion_request
                    .lock()
                    .await
                    .chat_history
                    .truncate(snapshot_len);
                return Err(AmbiError::MaxIterationsReached(agent.config.max_iterations));
            }

            let req_data = Agent::get_llm_request(
                &agent.completion_request,
                &agent.config.system_prompt,
                &agent.config.template,
                &agent.tools_def,
                &agent.cached_tool_prompt,
            )
            .await;

            let res = match engine.chat(req_data).await {
                Ok(r) => r,
                Err(e) => {
                    agent
                        .completion_request
                        .lock()
                        .await
                        .chat_history
                        .truncate(snapshot_len);
                    return Err(e);
                }
            };

            let dynamic_system_overhead: usize = agent
                .completion_request
                .lock()
                .await
                .chat_history
                .all()
                .iter()
                .filter(|m| matches!(***m, Message::System { .. }))
                .map(|m| m.estimate_tokens())
                .sum();

            let prompt_overhead =
                (agent.config.system_prompt.len() + agent.cached_tool_prompt.len()) / 4
                    + dynamic_system_overhead;

            let evicted_count = Self::append_assistant_message_and_evict(
                &agent.completion_request,
                res.clone(),
                &agent.on_evict_handler,
                agent.config.eviction_strategy,
                prompt_overhead,
            )
            .await;
            snapshot_len = snapshot_len.saturating_sub(evicted_count);

            let mut formatter: Box<dyn StreamFormatter> = if agent.config.enable_formatting {
                agent.tool_parser.create_stream_formatter()
            } else {
                Box::new(crate::agent::core::formatter::PassThroughFormatter)
            };

            final_formatted_output.push_str(&formatter.push(&res));
            final_formatted_output.push_str(&formatter.flush());

            let tool_calls = match Self::handle_tool_calls(
                &agent.completion_request,
                Arc::clone(&agent.tool_map),
                &agent.tool_parser,
                &res,
                None,
            )
            .await
            {
                Ok(calls) => calls,
                Err(e) => {
                    agent
                        .completion_request
                        .lock()
                        .await
                        .chat_history
                        .truncate(snapshot_len);
                    return Err(AmbiError::ToolError(e.to_string()));
                }
            };

            if tool_calls.is_empty() {
                return Ok(final_formatted_output.trim().to_string());
            }

            Self::process_tool_calls_output(&tool_calls, &mut final_formatted_output);
            iteration_count += 1;
        }
    }

    pub async fn chat_multimodal_stream(
        agent: &mut Agent,
        parts: Vec<ContentPart>,
    ) -> crate::error::Result<Pin<Box<ReceiverStream<crate::error::Result<String>>>>> {
        let mut owned_engine = match Arc::clone(&agent.llm_engine).try_lock_owned() {
            Ok(guard) => guard,
            Err(_) => return Err(AmbiError::AgentBusy),
        };

        let completion_request = Arc::clone(&agent.completion_request);
        let system_prompt = agent.config.system_prompt.clone();
        let parts_clone = parts.clone();

        let (tx_out, rx_out) = channel::<crate::error::Result<String>>(1024);

        let template_clone = agent.config.template.clone();
        let tool_map_clone = Arc::clone(&agent.tool_map);
        let tool_parser_clone = Arc::clone(&agent.tool_parser);
        let evict_handler_clone = agent.on_evict_handler.clone();
        let max_iterations = agent.config.max_iterations;
        let enable_formatting = agent.config.enable_formatting;
        let eviction_strategy = agent.config.eviction_strategy;
        let cached_tool_prompt = agent.cached_tool_prompt.clone();
        let tools_def_clone = Arc::clone(&agent.tools_def);

        tokio::spawn(async move {
            Self::append_user_message(&completion_request, parts_clone).await;
            let mut snapshot_len = completion_request.lock().await.chat_history.len();
            let mut iteration_count = 0;

            loop {
                if iteration_count >= max_iterations {
                    let _ = tx_out
                        .send(Err(AmbiError::AgentError("Max loops reached.".to_string())))
                        .await;
                    completion_request
                        .lock()
                        .await
                        .chat_history
                        .truncate(snapshot_len);
                    break;
                }

                let req_data = Agent::get_llm_request(
                    &completion_request,
                    &system_prompt,
                    &template_clone,
                    &tools_def_clone,
                    &cached_tool_prompt,
                )
                .await;

                let (tx_llm, rx_llm) = channel::<crate::error::Result<String>>(1024);

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

                let dynamic_system_overhead: usize = completion_request
                    .lock()
                    .await
                    .chat_history
                    .all()
                    .iter()
                    .filter(|m| matches!(***m, Message::System { .. }))
                    .map(|m| m.estimate_tokens())
                    .sum();

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
                        let _ = tx_out
                            .send(Err(AmbiError::ToolError(format!("Tool call error: {}", e))))
                            .await;
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
}
