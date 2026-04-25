// src/agent/pipeline/chat_runner/multimodal_handler.rs

use super::{ChatRunner, StateManager};
use crate::agent::core::{Agent, AgentState, EvictionHandler};
use crate::agent::tool::{DynTool, StreamFormatter, ToolCallParser, ToolDefinition};
use crate::error::AmbiError;
use crate::llm::{ChatTemplate, LLMEngine};
use crate::ContentPart;
use futures::FutureExt;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;

pub(crate) enum ExecutionMode<'a> {
    Sync,
    Stream {
        tx_out: &'a tokio::sync::mpsc::Sender<crate::error::Result<String>>,
        tool_parser: &'a Arc<dyn ToolCallParser>,
        enable_formatting: bool,
    },
}

pub(crate) struct LoopConfig<'a> {
    pub template: &'a ChatTemplate,
    pub max_iterations: usize,
    pub system_prompt: &'a str,
    pub eviction_strategy: (usize, usize, usize),
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
    pub tx_out: Option<&'a tokio::sync::mpsc::Sender<crate::error::Result<String>>>,
    pub evict_handler: &'a Option<EvictionHandler>,
}

impl ChatRunner {
    pub(crate) async fn chat_multimodal(
        agent: &Agent,
        state: &Arc<StdMutex<AgentState>>,
        parts: Vec<ContentPart>,
    ) -> crate::error::Result<String> {
        let accessor = StateManager(state);
        accessor.push_user_message(parts)?;

        let ctx = RunCtx {
            loop_config: LoopConfig {
                template: &agent.config.template,
                max_iterations: agent.config.max_iterations,
                system_prompt: &agent.config.system_prompt,
                eviction_strategy: agent.config.eviction_strategy,
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
        state: &Arc<StdMutex<AgentState>>,
        parts: Vec<ContentPart>,
    ) -> crate::error::Result<Pin<Box<ReceiverStream<crate::error::Result<String>>>>> {
        let (tx_out, rx_out) = channel::<crate::error::Result<String>>(1024);

        let agent_clone = agent.clone();
        let state_clone = Arc::clone(state);

        tokio::spawn(async move {
            let tx_out_clone = tx_out.clone();

            let task_logic = async move {
                let accessor = StateManager(&state_clone);
                if let Err(e) = accessor.push_user_message(parts) {
                    return Err(e);
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

                Self::run_loop(&ctx, &agent_clone.llm_engine, &accessor, mode).await
            };

            match AssertUnwindSafe(task_logic).catch_unwind().await {
                Ok(Err(e)) => {
                    let _ = tx_out.send(Err(e)).await;
                }
                Err(panic_err) => {
                    let msg = panic_err
                        .downcast_ref::<&str>()
                        .map(|s| s.to_string())
                        .or_else(|| panic_err.downcast_ref::<String>().cloned())
                        .unwrap_or_else(|| "Unknown internal panic".to_string());
                    log::error!("Pipeline streaming task panicked: {}", msg);
                    let _ = tx_out
                        .send(Err(AmbiError::PipelineError(format!(
                            "Framework panic: {}",
                            msg
                        ))))
                        .await;
                }
                _ => {}
            }
        });

        Ok(Box::pin(ReceiverStream::new(rx_out)))
    }

    pub(crate) async fn run_loop(
        ctx: &RunCtx<'_>,
        engine: &LLMEngine,
        accessor: &StateManager<'_>,
        mode: ExecutionMode<'_>,
    ) -> crate::error::Result<String> {
        let mut final_formatted_output = if ctx.tx_out.is_none() {
            String::with_capacity(2048)
        } else {
            String::new()
        };
        let mut iteration_count = 0;
        let mut snapshot_len = accessor.get_snapshot_len()?;

        loop {
            if iteration_count >= ctx.loop_config.max_iterations {
                accessor.truncate(snapshot_len)?;
                let err = AmbiError::MaxIterationsReached(ctx.loop_config.max_iterations);
                return if let Some(tx) = ctx.tx_out {
                    let _ = tx.send(Err(err)).await;
                    Ok(String::new())
                } else {
                    Err(err)
                };
            }

            let req_data = accessor.get_llm_request(
                ctx.loop_config.system_prompt,
                ctx.loop_config.template,
                ctx.loop_tooling.tools_def,
                ctx.loop_tooling.cached_tool_prompt,
            )?;

            let (full_output, has_error) = match &mode {
                ExecutionMode::Sync => match engine.chat(req_data).await {
                    Ok(res) => (res, false),
                    Err(e) => {
                        accessor.truncate(snapshot_len)?;
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
                accessor.truncate(snapshot_len)?;
                break;
            }

            let dynamic_system_overhead = accessor.get_system_overhead()?;
            let prompt_overhead = (ctx.loop_config.system_prompt.len()
                + ctx.loop_tooling.cached_tool_prompt.len())
                / 4
                + dynamic_system_overhead;

            let evicted_count = accessor.append_assistant_message_and_evict(
                full_output.clone(),
                ctx.evict_handler,
                ctx.loop_config.eviction_strategy,
                prompt_overhead,
            )?;
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
                Arc::clone(ctx.loop_tooling.tool_map),
                ctx.loop_tooling.tool_parser,
                &full_output,
                ctx.tx_out.cloned(),
            )
            .await
            {
                Ok(calls) => calls,
                Err(e) => {
                    accessor.truncate(snapshot_len)?;
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
