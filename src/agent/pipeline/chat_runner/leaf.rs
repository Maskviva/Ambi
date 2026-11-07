// src/agent/pipeline/chat_runner/leaf.rs

//! Feature-gated leaf sub-agent implementation for [`ChatRunner`].
//!
//! A leaf is a lightweight, non-recursive sub-agent mounted as a tool on [`ChatRunner`].
//! It reuses the parent agent's [`LLMEngine`], chat template, and pre-augmentation tool snapshot,
//! while executing inside a fresh, throwaway [`AgentState`] that is dropped upon completion.

use super::multimodal_handler::{ExecutionMode, LoopConfig, LoopTooling, ResolvedTooling, RunCtx};
use super::{ChatRunner, StateManager};
use crate::agent::core::{Agent, AgentState, DynToolObj, FormatterFactory, ToolCallParserObj};
use crate::config::EvictionStrategy;
use crate::error::{AmbiError, Result};
use crate::llm::LLMEngine;
use crate::types::{ChatTemplate, ContentPart, Message, Tool, ToolDefinition, ToolErr};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

static NEXT_LEAF_SESSION_ID: AtomicUsize = AtomicUsize::new(1);

/// Default model-facing tool name for the leaf sub-agent.
pub const DEFAULT_LEAF_TOOL_NAME: &str = "spawn_leaf";

/// Default timeout (in seconds) for a single leaf sub-agent invocation.
pub const DEFAULT_LEAF_TIMEOUT_SECS: u64 = 300;

/// Configuration for the leaf sub-agent capability on [`ChatRunner`].
///
/// Enabling the `sub-agent` Cargo feature alone does not alter [`ChatRunner::default`].
/// Call [`ChatRunner::with_leaf`] or [`ChatRunner::with_leaf_config`] to mount the leaf tool
/// for a runner instance.
///
/// # Architectural Notes & Constraints
///
/// - **Structural Non-Recursion**: The parent loop sees the augmented tool set (user tools + leaf tool),
///   whereas the leaf receives the pre-augmentation tool snapshot. A leaf can never see or invoke
///   `spawn_leaf`, making recursive spawning impossible by construction.
/// - **State Isolation vs. Engine Sharing**: Each leaf invocation allocates a fresh [`AgentState`] with
///   a distinct `session_id` and `evict_handler = None`, then drops that state when the tool call returns.
///   However, the underlying [`LLMEngine`] is shared with the parent. On local `llama.cpp` backends
///   (single worker thread + single KV cache), leaf calls serialize on the worker queue and switch the
///   active KV cache session.
/// - **Inherited Tools**: A leaf inherits all tools registered on the parent [`Agent`], including their
///   side effects.
#[derive(Clone, Debug)]
pub struct LeafConfig {
    /// Model-facing name of the injected leaf tool. Defaults to `"spawn_leaf"`.
    pub tool_name: String,
    /// Model-facing description instructing the parent LLM when and how to use the leaf tool.
    pub description: String,
    /// Optional system prompt override for the leaf. When `None`, inherits the parent agent's `system_prompt`.
    pub system_prompt: Option<String>,
    /// Optional maximum ReAct iterations per leaf call. When `None`, inherits the parent agent's `max_iterations`.
    pub max_iterations: Option<usize>,
    /// Optional maximum concurrent tool calls inside a single leaf loop.
    /// When `None`, inherits the parent [`ChatRunner::maximum_concurrency`].
    pub maximum_concurrency: Option<usize>,
    /// Maximum execution time in seconds for a single leaf invocation. Defaults to `300`.
    pub timeout_secs: u64,
}

impl Default for LeafConfig {
    fn default() -> Self {
        Self {
            tool_name: DEFAULT_LEAF_TOOL_NAME.to_string(),
            description: "Delegate an independent, self-contained subtask to an isolated leaf sub-agent. \
                The leaf shares your tools and model, starts with a clean conversation state, and returns \
                only its final answer. Emit multiple calls in the same turn to run independent subtasks in parallel."
                .to_string(),
            system_prompt: None,
            max_iterations: None,
            maximum_concurrency: None,
            timeout_secs: DEFAULT_LEAF_TIMEOUT_SECS,
        }
    }
}

impl LeafConfig {
    /// Creates a new [`LeafConfig`] with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the model-facing tool name (default: `"spawn_leaf"`).
    pub fn with_tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = tool_name.into();
        self
    }

    /// Sets the model-facing tool description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Overrides the system prompt used by leaf sub-agents.
    pub fn with_system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    /// Overrides the maximum number of ReAct iterations allowed per leaf execution.
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }

    /// Overrides the maximum tool concurrency inside a single leaf execution.
    pub fn with_maximum_concurrency(mut self, maximum_concurrency: usize) -> Self {
        self.maximum_concurrency = Some(maximum_concurrency);
        self
    }

    /// Sets the timeout in seconds for a single leaf execution (default: `300`).
    pub fn with_timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub(crate) fn validate(&self, agent: &Agent) -> Result<()> {
        if self.tool_name.trim().is_empty() {
            return Err(AmbiError::PipelineError(
                "LeafConfig tool_name cannot be empty".into(),
            ));
        }
        if self.max_iterations == Some(0) {
            return Err(AmbiError::PipelineError(
                "LeafConfig max_iterations must be greater than 0".into(),
            ));
        }
        if self.maximum_concurrency == Some(0) {
            return Err(AmbiError::PipelineError(
                "LeafConfig maximum_concurrency must be greater than 0".into(),
            ));
        }
        if self.timeout_secs == 0 {
            return Err(AmbiError::PipelineError(
                "LeafConfig timeout_secs must be greater than 0".into(),
            ));
        }
        if agent.tool_map.contains_key(&self.tool_name)
            || agent.tools_def.iter().any(|t| t.name == self.tool_name)
        {
            return Err(AmbiError::AgentError(format!(
                "Tool registration conflict: Leaf tool name '{}' collides with an already registered tool.",
                self.tool_name
            )));
        }
        Ok(())
    }
}

/// Pre-augmentation runtime snapshot held by a leaf sub-agent.
#[derive(Clone)]
pub(crate) struct Leaf {
    llm_engine: Arc<LLMEngine>,
    template: ChatTemplate,
    max_iterations: usize,
    system_prompt: String,
    eviction_strategy: EvictionStrategy,
    maximum_concurrency: usize,
    tools_def: Arc<Vec<ToolDefinition>>,
    cached_tool_prompt: Arc<String>,
    tool_map: Arc<HashMap<String, Arc<DynToolObj>>>,
    tool_parser: Arc<ToolCallParserObj>,
    formatter_factory: FormatterFactory,
}

impl Leaf {
    fn from_parent(agent: &Agent, runner_concurrency: usize, cfg: &LeafConfig) -> Self {
        Self {
            llm_engine: Arc::clone(&agent.llm_engine),
            template: agent.config.template.clone(),
            max_iterations: cfg.max_iterations.unwrap_or(agent.config.max_iterations),
            system_prompt: cfg
                .system_prompt
                .clone()
                .unwrap_or_else(|| agent.config.system_prompt.clone()),
            eviction_strategy: agent.config.eviction_strategy.clone(),
            maximum_concurrency: cfg.maximum_concurrency.unwrap_or(runner_concurrency),
            // Pre-augmentation snapshot: does NOT contain `LeafTool`, preventing recursion.
            tools_def: Arc::clone(&agent.tools_def),
            cached_tool_prompt: Arc::clone(&agent.cached_tool_prompt),
            tool_map: Arc::clone(&agent.tool_map),
            tool_parser: Arc::clone(&agent.tool_parser),
            formatter_factory: Arc::clone(&agent.formatter_factory),
        }
    }

    async fn execute(
        &self,
        task: String,
        context: Option<String>,
    ) -> std::result::Result<String, ToolErr> {
        if task.trim().is_empty() {
            return Err(ToolErr("Leaf task cannot be empty.".to_string()));
        }

        let leaf_id = NEXT_LEAF_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        let state = AgentState::new_shared(format!("leaf-session-{}", leaf_id));

        if let Some(ctx) = context {
            let trimmed = ctx.trim();
            if !trimmed.is_empty() {
                state.write().await.set_dynamic_context(trimmed);
            }
        }

        let parts = vec![ContentPart::Text { text: task }];
        let tokens = self
            .llm_engine
            .count_tokens(
                &Message::User {
                    content: parts.clone(),
                }
                .to_string(),
            )
            .map_err(|e| ToolErr(e.to_string()))?;

        let accessor = StateManager(&state);
        accessor
            .push_user_message(parts, tokens)
            .await
            .map_err(|e| ToolErr(e.to_string()))?;

        let no_evict_handler = None;
        let run_ctx = RunCtx {
            loop_config: LoopConfig {
                template: &self.template,
                max_iterations: self.max_iterations,
                system_prompt: &self.system_prompt,
                eviction_strategy: self.eviction_strategy.clone(),
            },
            loop_tooling: LoopTooling {
                tools_def: &self.tools_def,
                cached_tool_prompt: &self.cached_tool_prompt,
                tool_map: &self.tool_map,
                tool_parser: &self.tool_parser,
                formatter_factory: &self.formatter_factory,
            },
            tx_out: None,
            evict_handler: &no_evict_handler,
            final_answer_only: true,
        };

        match ChatRunner::run_loop(
            &run_ctx,
            &self.llm_engine,
            self.maximum_concurrency,
            &accessor,
            ExecutionMode::Sync,
        )
        .await
        {
            Ok(answer) => Ok(answer),
            Err(AmbiError::MaxIterationsReached(limit)) => Err(ToolErr(format!(
                "Leaf sub-agent reached its maximum iteration limit ({}) without completing the task. \
                 Please narrow or decompose the task into smaller steps.",
                limit
            ))),
            Err(e) => Err(ToolErr(format!("Leaf sub-agent execution failed: {}", e))),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct LeafArgs {
    pub task: String,
    #[serde(default)]
    pub context: Option<String>,
}

pub(crate) struct LeafTool {
    tool_name: String,
    description: String,
    timeout_secs: u64,
    leaf: Leaf,
}

impl LeafTool {
    pub(crate) fn new(agent: &Agent, runner_concurrency: usize, cfg: &LeafConfig) -> Self {
        Self {
            tool_name: cfg.tool_name.clone(),
            description: cfg.description.clone(),
            timeout_secs: cfg.timeout_secs,
            leaf: Leaf::from_parent(agent, runner_concurrency, cfg),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for LeafTool {
    const NAME: &'static str = DEFAULT_LEAF_TOOL_NAME;
    type Args = LeafArgs;
    type Output = String;

    fn name(&self) -> String {
        self.tool_name.clone()
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.tool_name.clone(),
            description: self.description.clone(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": "A clear, self-contained instruction for the leaf sub-agent to complete."
                    },
                    "context": {
                        "type": "string",
                        "description": "Optional background context or constraints needed for this subtask (the leaf does not see parent chat history)."
                    }
                },
                "required": ["task"]
            }),
            timeout_secs: Some(self.timeout_secs),
            max_retries: Some(0),
            is_idempotent: false,
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, ToolErr> {
        self.leaf.execute(args.task, args.context).await
    }
}

#[cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]
pub(crate) fn build_augmented_tooling(
    agent: &Agent,
    runner_concurrency: usize,
    cfg: &LeafConfig,
) -> Result<ResolvedTooling> {
    cfg.validate(agent)?;

    let leaf_tool = LeafTool::new(agent, runner_concurrency, cfg);
    let leaf_def = leaf_tool.definition();

    let mut tools_def = (*agent.tools_def).clone();
    tools_def.push(leaf_def.clone());
    let tools_def = Arc::new(tools_def);

    let mut tool_map = (*agent.tool_map).clone();
    let dyn_leaf: Arc<DynToolObj> = Arc::new(leaf_tool);
    tool_map.insert(leaf_def.name, dyn_leaf);
    let tool_map = Arc::new(tool_map);

    let tools_json = serde_json::to_string(&*tools_def).unwrap_or_default();
    let cached_tool_prompt = Arc::new(agent.tool_parser.format_instruction(&tools_json));

    Ok(ResolvedTooling {
        tools_def,
        cached_tool_prompt,
        tool_map,
        tool_parser: Arc::clone(&agent.tool_parser),
        formatter_factory: Arc::clone(&agent.formatter_factory),
    })
}
