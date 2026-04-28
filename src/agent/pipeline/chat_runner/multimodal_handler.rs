// src/agent/pipeline/chat_runner/multimodal_handler.rs

use super::{ChatRunner, StateManager};
use crate::agent::core::{Agent, AgentState, EvictionHandler};
use crate::agent::tool::{DynTool, StreamFormatter, ToolCallParser, ToolDefinition};
use crate::error::{AmbiError, Result};
use crate::llm::{ChatTemplate, LLMEngine};
use crate::types::config::EvictionStrategy;
use crate::types::message::Message;
use crate::ContentPart;

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

pub(crate) enum ExecutionMode<'a> {
    Sync,
    Stream {
        tx_out: &'a tokio::sync::mpsc::Sender<Result<String>>,
        tool_parser: &'a Arc<dyn ToolCallParser>,
        enable_formatting: bool,
    },
}

pub(crate) struct LoopConfig<'a> {
    pub template: &'a ChatTemplate,
    pub max_iterations: usize,
    pub system_prompt: &'a str,
    pub eviction_strategy: EvictionStrategy,
    pub enable_formatting: bool,
}

pub(crate) struct LoopTooling<'a> {
    pub tools_def: &'a Arc<Vec<ToolDefinition>>,
    pub cached_tool_prompt: &'a str,
    pub tool_map: &'a Arc<HashMap<String, Arc<dyn DynTool>>>,
    pub tool_parser: &'a Arc<dyn ToolCallParser>,
}

pub(crate) struct RunCtx<'a> {
    pub loop_config: LoopConfig<'a>,
    pub loop_tooling: LoopTooling<'a>,
    pub tx_out: Option<&'a tokio::sync::mpsc::Sender<Result<String>>>,
    pub evict_handler: &'a Option<EvictionHandler>,
}

impl ChatRunner {
    pub(crate) async fn chat_multimodal(
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        parts: Vec<ContentPart>,
    ) -> Result<String> {
        let has_image = parts.iter().any(|p| matches!(p, ContentPart::Image { .. }));
        if has_image && !agent.llm_engine.supports_multimodal() {
            return Err(AmbiError::EngineError(
                "Security Check Failed: The current LLM engine does not support multimodal (image) inputs.".into()
            ));
        }

        let user_msg = Message::User {
            content: parts.clone(),
        };
        let tokens = agent.llm_engine.count_tokens(&user_msg.to_string())?;

        let accessor = StateManager(state);
        accessor.push_user_message(parts, tokens).await?;

        let ctx = RunCtx {
            loop_config: LoopConfig {
                template: &agent.config.template,
                max_iterations: agent.config.max_iterations,
                system_prompt: &agent.config.system_prompt,
                eviction_strategy: agent.config.eviction_strategy.clone(),
                enable_formatting: agent.config.enable_formatting,
            },
            loop_tooling: LoopTooling {
                tools_def: &agent.tools_def,
                cached_tool_prompt: &agent.cached_tool_prompt,
                tool_map: &agent.tool_map,
                tool_parser: &agent.tool_parser,
            },
            tx_out: None,
            evict_handler: &agent.on_evict_handler,
        };

        Self::run_loop(&ctx, &agent.llm_engine, &accessor, ExecutionMode::Sync).await
    }

    pub(crate) async fn chat_multimodal_stream(
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        parts: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        let has_image = parts.iter().any(|p| matches!(p, ContentPart::Image { .. }));
        if has_image && !agent.llm_engine.supports_multimodal() {
            return Err(AmbiError::EngineError(
                "Security Check Failed: The current LLM engine does not support multimodal (image) inputs.".into()
            ));
        }

        let (tx_out, rx_out) = channel::<Result<String>>(1024);

        let tx_out_for_panic = tx_out.clone();

        let agent_clone = agent.clone();
        let state_clone = Arc::clone(state);

        let user_msg = Message::User {
            content: parts.clone(),
        };
        let tokens = agent_clone.llm_engine.count_tokens(&user_msg.to_string())?;

        let handle = tokio::spawn(async move {
            let tx_out_clone = tx_out.clone();

            let accessor = StateManager(&state_clone);

            if let Err(e) = accessor.push_user_message(parts, tokens).await {
                let _ = tx_out_clone.send(Err(e)).await;
                return;
            }

            let ctx = RunCtx {
                loop_config: LoopConfig {
                    template: &agent_clone.config.template,
                    max_iterations: agent_clone.config.max_iterations,
                    system_prompt: &agent_clone.config.system_prompt,
                    eviction_strategy: agent_clone.config.eviction_strategy,
                    enable_formatting: agent_clone.config.enable_formatting,
                },
                loop_tooling: LoopTooling {
                    tools_def: &agent_clone.tools_def,
                    cached_tool_prompt: &agent_clone.cached_tool_prompt,
                    tool_map: &agent_clone.tool_map,
                    tool_parser: &agent_clone.tool_parser,
                },
                tx_out: Some(&tx_out_clone),
                evict_handler: &agent_clone.on_evict_handler,
            };

            let mode = ExecutionMode::Stream {
                tx_out: &tx_out_clone,
                tool_parser: &agent_clone.tool_parser,
                enable_formatting: agent_clone.config.enable_formatting,
            };

            if let Err(e) = Self::run_loop(&ctx, &agent_clone.llm_engine, &accessor, mode).await {
                let _ = tx_out_clone.send(Err(e)).await;
            }
        });

        tokio::spawn(async move {
            if let Err(join_err) = handle.await {
                if join_err.is_panic() {
                    log::error!("CRITICAL: Pipeline streaming task panicked internally.");
                    let _ = tx_out_for_panic
                        .send(Err(AmbiError::PipelineError(
                            "Internal framework panic caught, avoiding process crash.".into(),
                        )))
                        .await;
                }
            }
        });

        Ok(Box::pin(ReceiverStream::new(rx_out)))
    }

    pub(crate) async fn run_loop(
        ctx: &RunCtx<'_>,
        engine: &LLMEngine,
        accessor: &StateManager<'_>,
        mode: ExecutionMode<'_>,
    ) -> Result<String> {
        let mut final_formatted_output = if ctx.tx_out.is_none() {
            String::with_capacity(2048)
        } else {
            String::new()
        };
        let mut iteration_count = 0;
        let mut snapshot_len = accessor.get_snapshot_len().await?;

        loop {
            if iteration_count >= ctx.loop_config.max_iterations {
                accessor.truncate(snapshot_len).await?;
                let err = AmbiError::MaxIterationsReached(ctx.loop_config.max_iterations);
                return if let Some(tx) = ctx.tx_out {
                    let _ = tx.send(Err(err)).await;
                    Ok(String::new())
                } else {
                    Err(err)
                };
            }

            let req_data = accessor
                .get_llm_request(
                    ctx.loop_config.system_prompt,
                    ctx.loop_config.template,
                    ctx.loop_tooling.tools_def,
                    ctx.loop_tooling.cached_tool_prompt,
                    ctx.loop_tooling.tool_parser.get_tags(),
                )
                .await?;

            let (full_output, has_error) = match &mode {
                ExecutionMode::Sync => match engine.chat(req_data).await {
                    Ok(res) => (res, false),
                    Err(e) => {
                        accessor.truncate(snapshot_len).await?;
                        return Err(e);
                    }
                },
                ExecutionMode::Stream {
                    tx_out,
                    tool_parser,
                    enable_formatting,
                } => {
                    let (tx_llm, rx_llm) = channel::<Result<String>>(1024);
                    let process_future =
                        Self::process_llm_stream(rx_llm, tx_out, tool_parser, *enable_formatting);
                    let engine_future = engine.chat_stream(req_data, tx_llm);
                    let (full_output, stream_error) = tokio::join!(engine_future, process_future).1;

                    if let Some(err) = stream_error {
                        accessor.truncate(snapshot_len).await?;
                        return Err(err);
                    }
                    (full_output, false)
                }
            };

            if has_error {
                accessor.truncate(snapshot_len).await?;
                break;
            }

            let parsed_tool_calls = ctx.loop_tooling.tool_parser.parse(&full_output);
            let tool_calls_with_ids: Vec<_> = parsed_tool_calls
                .into_iter()
                .enumerate()
                .map(|(i, (name, args))| {
                    let id = format!(
                        "call_ambi_{}_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                        i
                    );
                    (name, args, id)
                })
                .collect();

            let asst_msg = Message::Assistant {
                content: full_output.clone(),
                tool_calls: tool_calls_with_ids.clone(),
            };
            let tokens = engine.count_tokens(&asst_msg.to_string())?;

            let dynamic_system_overhead = accessor.get_system_overhead().await?;
            let prompt_overhead = engine.count_tokens(ctx.loop_config.system_prompt)?
                + engine.count_tokens(ctx.loop_tooling.cached_tool_prompt)?
                + dynamic_system_overhead;

            let evicted_count = accessor
                .append_assistant_message_and_evict(
                    full_output.clone(),
                    tool_calls_with_ids.clone(),
                    tokens,
                    ctx.evict_handler,
                    &ctx.loop_config.eviction_strategy,
                    prompt_overhead,
                )
                .await?;
            snapshot_len = snapshot_len.saturating_sub(evicted_count);

            if ctx.tx_out.is_none() {
                let mut formatter: Box<dyn StreamFormatter> = if ctx.loop_config.enable_formatting {
                    ctx.loop_tooling.tool_parser.create_stream_formatter()
                } else {
                    Box::new(crate::agent::core::formatter::PassThroughFormatter)
                };
                final_formatted_output.push_str(&formatter.push(&full_output));
                final_formatted_output.push_str(&formatter.flush());
            }

            let tool_calls = match Self::handle_tool_calls(
                accessor,
                engine,
                Arc::clone(ctx.loop_tooling.tool_map),
                tool_calls_with_ids,
                ctx.tx_out.cloned(),
            )
            .await
            {
                Ok(calls) => calls,
                Err(e) => {
                    accessor.truncate(snapshot_len).await?;
                    return if let Some(tx) = ctx.tx_out {
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

            if let Some(tx) = ctx.tx_out {
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
