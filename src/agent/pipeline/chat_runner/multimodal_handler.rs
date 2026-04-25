// src/agent/pipeline/chat_runner/multimodal_handler.rs

use crate::agent::core::{CompletionRequest, EvictionHandler};
use crate::agent::tool::{DynTool, StreamFormatter, ToolCallParser, ToolDefinition};
use crate::error::AmbiError;
use crate::llm::{ChatTemplate, LLMEngine};
use crate::{Agent, ChatRunner, ContentPart, Message};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::wrappers::ReceiverStream;

enum ExecutionMode<'a> {
    Sync,
    Stream {
        tx_out: &'a tokio::sync::mpsc::Sender<crate::error::Result<String>>,
        tool_parser: &'a Arc<dyn ToolCallParser>,
        enable_formatting: bool,
    },
}

struct LoopConfig<'a> {
    template: &'a ChatTemplate,
    max_iterations: usize,
    system_prompt: &'a str,
    eviction_strategy: (usize, usize, usize),
    enable_formatting: bool,
}

struct LoopSharedRefs<'a> {
    completion_request: &'a Arc<TokioMutex<CompletionRequest>>,
    tx_out: Option<&'a tokio::sync::mpsc::Sender<crate::error::Result<String>>>,
    evict_handler: &'a Option<EvictionHandler>,
}

struct LoopTooling<'a> {
    tools_def: &'a Arc<Vec<ToolDefinition>>,
    cached_tool_prompt: &'a str,
    tool_map: &'a Arc<HashMap<String, Arc<dyn DynTool>>>,
    tool_parser: &'a Arc<dyn ToolCallParser>,
}

struct RunCtx<'a> {
    loop_config: LoopConfig<'a>,
    loop_shared_refs: LoopSharedRefs<'a>,
    loop_tooling: LoopTooling<'a>,
}

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

        let ctx = RunCtx {
            loop_config: LoopConfig {
                template: &agent.config.template,
                max_iterations: agent.config.max_iterations,
                system_prompt: &agent.config.system_prompt,
                eviction_strategy: agent.config.eviction_strategy,
                enable_formatting: agent.config.enable_formatting,
            },
            loop_shared_refs: LoopSharedRefs {
                completion_request: &agent.completion_request,
                tx_out: None,
                evict_handler: &agent.on_evict_handler,
            },
            loop_tooling: LoopTooling {
                tools_def: &agent.tools_def,
                cached_tool_prompt: &agent.cached_tool_prompt,
                tool_map: &agent.tool_map,
                tool_parser: &agent.tool_parser,
            },
        };

        Self::run_loop(&ctx, &mut engine, ExecutionMode::Sync).await
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
        let template = agent.config.template.clone();
        let tools_def = Arc::clone(&agent.tools_def);
        let cached_tool_prompt = agent.cached_tool_prompt.clone();
        let tool_map = Arc::clone(&agent.tool_map);
        let tool_parser = Arc::clone(&agent.tool_parser);
        let evict_handler = agent.on_evict_handler.clone();
        let max_iterations = agent.config.max_iterations;
        let enable_formatting = agent.config.enable_formatting;
        let eviction_strategy = agent.config.eviction_strategy;

        let (tx_out, rx_out) = channel::<crate::error::Result<String>>(1024);

        tokio::spawn(async move {
            Self::append_user_message(&completion_request, parts).await;

            let ctx = RunCtx {
                loop_config: LoopConfig {
                    template: &template,
                    max_iterations,
                    system_prompt: &system_prompt,
                    eviction_strategy,
                    enable_formatting,
                },
                loop_shared_refs: LoopSharedRefs {
                    completion_request: &completion_request,
                    tx_out: None,
                    evict_handler: &evict_handler,
                },
                loop_tooling: LoopTooling {
                    tools_def: &tools_def,
                    cached_tool_prompt: &cached_tool_prompt,
                    tool_map: &tool_map,
                    tool_parser: &tool_parser,
                },
            };

            let mode = ExecutionMode::Stream {
                tx_out: &tx_out,
                tool_parser: &tool_parser,
                enable_formatting,
            };

            let _ = Self::run_loop(&ctx, &mut owned_engine, mode).await;
        });

        Ok(Box::pin(ReceiverStream::new(rx_out)))
    }

    async fn run_loop(
        ctx: &RunCtx<'_>,
        engine: &mut LLMEngine,
        mode: ExecutionMode<'_>,
    ) -> crate::error::Result<String> {
        let mut final_formatted_output = if ctx.loop_shared_refs.tx_out.is_none() {
            String::with_capacity(2048)
        } else {
            String::new()
        };

        let mut iteration_count = 0;
        let mut snapshot_len = ctx
            .loop_shared_refs
            .completion_request
            .lock()
            .await
            .chat_history
            .len();

        loop {
            if iteration_count >= ctx.loop_config.max_iterations {
                ctx.loop_shared_refs
                    .completion_request
                    .lock()
                    .await
                    .chat_history
                    .truncate(snapshot_len);

                let err = AmbiError::MaxIterationsReached(ctx.loop_config.max_iterations);
                return if let Some(tx) = ctx.loop_shared_refs.tx_out {
                    let _ = tx.send(Err(err)).await;
                    Ok(String::new())
                } else {
                    Err(err)
                };
            }

            let req_data = Agent::get_llm_request(
                ctx.loop_shared_refs.completion_request,
                ctx.loop_config.system_prompt,
                ctx.loop_config.template,
                ctx.loop_tooling.tools_def,
                ctx.loop_tooling.cached_tool_prompt,
            )
            .await;

            let (full_output, has_error) = match &mode {
                ExecutionMode::Sync => match engine.chat(req_data).await {
                    Ok(res) => (res, false),
                    Err(e) => {
                        ctx.loop_shared_refs
                            .completion_request
                            .lock()
                            .await
                            .chat_history
                            .truncate(snapshot_len);
                        return Err(e);
                    }
                },
                ExecutionMode::Stream {
                    tx_out,
                    tool_parser,
                    enable_formatting,
                } => {
                    let (tx_llm, rx_llm) = channel::<crate::error::Result<String>>(1024);
                    let process_future =
                        Self::process_llm_stream(rx_llm, tx_out, tool_parser, *enable_formatting);
                    let engine_future = engine.chat_stream(req_data, tx_llm);

                    tokio::join!(engine_future, process_future).1
                }
            };

            if has_error {
                ctx.loop_shared_refs
                    .completion_request
                    .lock()
                    .await
                    .chat_history
                    .truncate(snapshot_len);
                break;
            }

            let guard = ctx.loop_shared_refs.completion_request.lock().await;
            let dynamic_system_overhead: usize = {
                guard
                    .chat_history
                    .all()
                    .iter()
                    .filter(|m| matches!(***m, Message::System { .. }))
                    .map(|m| m.estimate_tokens())
                    .sum()
            };

            let prompt_overhead = (ctx.loop_config.system_prompt.len()
                + ctx.loop_tooling.cached_tool_prompt.len())
                / 4
                + dynamic_system_overhead;

            let evicted_count = Self::append_assistant_message_and_evict(
                ctx.loop_shared_refs.completion_request,
                full_output.clone(),
                ctx.loop_shared_refs.evict_handler,
                ctx.loop_config.eviction_strategy,
                prompt_overhead,
            )
            .await;
            snapshot_len = snapshot_len.saturating_sub(evicted_count);

            if ctx.loop_shared_refs.tx_out.is_none() {
                let mut formatter: Box<dyn StreamFormatter> = if ctx.loop_config.enable_formatting {
                    ctx.loop_tooling.tool_parser.create_stream_formatter()
                } else {
                    Box::new(crate::agent::core::formatter::PassThroughFormatter)
                };

                final_formatted_output.push_str(&formatter.push(&full_output));
                final_formatted_output.push_str(&formatter.flush());
            }

            let tool_calls = match Self::handle_tool_calls(
                ctx.loop_shared_refs.completion_request,
                Arc::clone(ctx.loop_tooling.tool_map),
                ctx.loop_tooling.tool_parser,
                &full_output,
                ctx.loop_shared_refs.tx_out.cloned(),
            )
            .await
            {
                Ok(calls) => calls,
                Err(e) => {
                    ctx.loop_shared_refs
                        .completion_request
                        .lock()
                        .await
                        .chat_history
                        .truncate(snapshot_len);
                    return if let Some(tx) = ctx.loop_shared_refs.tx_out {
                        let _ = tx.send(Err(AmbiError::ToolError(e.to_string()))).await;
                        Ok(String::new())
                    } else {
                        Err(AmbiError::ToolError(e.to_string()))
                    };
                }
            };

            if tool_calls.is_empty() {
                break;
            }

            if let Some(tx) = ctx.loop_shared_refs.tx_out {
                let mut formatted_tools = String::with_capacity(1024);
                Self::process_tool_calls_output(&tool_calls, &mut formatted_tools);
                let _ = tx.send(Ok(formatted_tools)).await;
            } else {
                Self::process_tool_calls_output(&tool_calls, &mut final_formatted_output);
            }

            iteration_count += 1;
        }

        Ok(final_formatted_output.trim().to_string())
    }
}
