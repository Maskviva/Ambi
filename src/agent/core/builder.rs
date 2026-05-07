// src/agent/core/builder.rs

use super::{Agent, ExtensionsMap};
use crate::agent::processor::{PassThroughFormatter, StandardStreamFormatter};
use crate::agent::tool::DefaultToolParser;
use crate::config::{AgentConfig, EvictionStrategy};
use crate::error::{AmbiError, Result};
use crate::llm::{LLMEngine, LLMEngineConfig, LLMEngineTrait};
use crate::runtime::spawn_blocking;
use crate::types::{ChatTemplate, Message, StreamFormatter, Tool, ToolCallParser, ToolDefinition};

use crate::agent::core::history::ChatHistory;
use crate::AgentState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

impl AgentState {
    /// Creates a fresh, empty conversation state.
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            dynamic_context: String::new(),
            chat_history: ChatHistory::new(),
            extensions: ExtensionsMap::new(),
        }
    }

    /// Convenience: returns a thread-shared, lock-protected AgentState,
    /// ready to be passed directly into Pipeline::execute / execute_stream.
    #[cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]
    pub fn new_shared(session_id: impl Into<String>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new(session_id)))
    }

    /// Get a mutable reference to the expanded mapping.
    pub fn extensions_mut(&mut self) -> &mut ExtensionsMap {
        &mut self.extensions
    }

    /// Get an immutable reference to the extended mapping.
    pub fn extensions(&self) -> &ExtensionsMap {
        &self.extensions
    }
}

impl Agent {
    /// # Constructors
    /// Creates a new Agent using a standard, framework-supported `LLMEngineConfig`.
    pub async fn make(engine_cfg: LLMEngineConfig) -> Result<Self> {
        let engine = spawn_blocking(move || LLMEngine::load(engine_cfg))
            .await
            .map_err(|e| {
                AmbiError::EngineError(format!("Failed to spawn blocking task: {}", e))
            })??;

        Ok(Self::init_agent(engine))
    }

    /// Creates an Agent using a custom LLM backend via the `LLMEngineConfig::Custom` variant.
    ///
    /// # Deprecation
    /// This method is deprecated. Use `Agent::make(LLMEngineConfig::Custom(backend)).await` instead.
    ///
    /// # Migration
    ///
    /// ```rust,ignore
    /// // Old (deprecated):
    /// let agent = Agent::with_custom_engine(Box::new(MyEngine))?;
    ///
    /// // New (recommended):
    /// let agent = Agent::make(LLMEngineConfig::Custom(Box::new(MyEngine))).await?;
    /// ```
    #[deprecated(
        since = "0.3.3",
        note = "use `Agent::make(LLMEngineConfig::Custom(backend)).await` instead"
    )]
    #[allow(deprecated)]
    pub fn with_custom_engine(custom_backend: Box<dyn LLMEngineTrait>) -> Result<Self> {
        let engine = LLMEngine::from_custom(custom_backend)?;
        Ok(Self::init_agent(engine))
    }

    /// Creates an Agent from an already-initialized Engine (Arc).
    ///
    /// This is the primitive constructor; `init_agent` and `make` both go through this.
    #[cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]
    pub fn from_engine(llm_engine: Arc<LLMEngine>) -> Self {
        Self {
            llm_engine,
            config: Arc::new(AgentConfig::default()),
            tools_def: Arc::new(Vec::new()),
            tool_map: Arc::new(HashMap::new()),
            tool_parser: Arc::new(DefaultToolParser::make()),
            on_evict_handler: None,
            formatter_factory: Arc::new(|| Box::new(PassThroughFormatter)),
            cached_tool_prompt: Arc::new(String::new()),
        }
    }

    #[cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]
    pub(super) fn init_agent(engine: LLMEngine) -> Self {
        Self::from_engine(Arc::new(engine))
    }

    /// # Core Configuration
    /// Sets the persistent global system prompt (instructions) for the Agent.
    pub fn preamble(mut self, system_prompt: &str) -> Self {
        Arc::make_mut(&mut self.config).system_prompt = system_prompt.to_string();
        self
    }

    /// Configures the chat template format (e.g., ChatML, Llama3) used to compile prompts.
    pub fn template<T: Into<ChatTemplate>>(mut self, template_source: T) -> Self {
        Arc::make_mut(&mut self.config).template = template_source.into();
        self
    }

    /// Customizes the contextual memory limits and eviction behavior.
    pub fn with_eviction_strategy(mut self, strategy: EvictionStrategy) -> Self {
        Arc::make_mut(&mut self.config).eviction_strategy = strategy;
        self
    }

    /// # Tooling Assembly
    /// Registers a custom tool to the Agent.
    /// Fails fast and returns an Error if a tool with the same name already exists.
    pub fn tool<T: Tool + 'static>(mut self, tool: T) -> Result<Self> {
        let def = tool.definition();

        let defs = Arc::make_mut(&mut self.tools_def);
        let map = Arc::make_mut(&mut self.tool_map);

        if defs.iter().any(|t| t.name == def.name) {
            return Err(AmbiError::AgentError(format!(
                "Tool registration conflict: A tool named '{}' is already registered. \
                 Please rename your tool or handle the conflict in your setup logic.",
                def.name
            )));
        }

        defs.push(ToolDefinition {
            name: def.name.clone(),
            description: def.description,
            parameters: def.parameters,
            timeout_secs: def.timeout_secs,
            max_retries: def.max_retries,
            is_idempotent: def.is_idempotent,
        });
        map.insert(def.name, Arc::new(tool));

        self.update_cached_tool_prompt();
        Ok(self)
    }

    /// Register multiple custom tools with the agent at the same time.
    pub fn with_dyn_tools<T: Tool + 'static>(mut self, tools: Vec<Arc<T>>) -> Result<Self> {
        for tool in tools {
            let def = tool.definition();
            Arc::make_mut(&mut self.tools_def).push(def.clone());
            Arc::make_mut(&mut self.tool_map).insert(def.name, tool);
        }
        self.update_cached_tool_prompt();
        Ok(self)
    }

    /// Injects a custom parser to define how the LLM outputs tool-call requests.
    pub fn with_tool_parser<P: ToolCallParser + 'static>(mut self, parser: P) -> Self {
        self.tool_parser = Arc::new(parser);

        self.update_cached_tool_prompt();
        self
    }

    fn update_cached_tool_prompt(&mut self) {
        if self.tools_def.is_empty() {
            self.cached_tool_prompt = Arc::new(String::new());
        } else {
            let tools_json = serde_json::to_string(&*self.tools_def).unwrap_or_default();
            self.cached_tool_prompt = Arc::new(self.tool_parser.format_instruction(&tools_json));
        }
    }

    // --- Processors & Lifecycle Hooks ---

    /// Injects a custom stream formatter factory to manipulate raw LLM output text on the fly.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_stream_formatter<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> Box<dyn StreamFormatter + Send + Sync> + Send + Sync + 'static,
    {
        self.formatter_factory = Arc::new(factory);
        self
    }

    /// Injects a custom stream formatter factory to manipulate raw LLM output text on the fly.
    #[cfg(target_arch = "wasm32")]
    pub fn with_stream_formatter<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> Box<dyn StreamFormatter> + 'static,
    {
        self.formatter_factory = Arc::new(factory);
        self
    }

    /// Syntactic sugar: applies the framework's `StandardStreamFormatter`.
    /// Automatically intercepts tool call syntaxes and formats reasoning (think) blocks cleanly.
    pub fn with_standard_formatting(mut self) -> Self {
        let (tool_start, tool_end) = self.tool_parser.get_tags();
        let think_start = self.config.template.think_prefix.clone();
        let think_end = self.config.template.think_suffix.clone();

        self.formatter_factory = Arc::new(move || {
            Box::new(StandardStreamFormatter::new(
                &tool_start,
                &tool_end,
                &think_start,
                &think_end,
            ))
        });
        self
    }

    /// Registers a callback that triggers whenever conversation history is evicted
    /// due to token capacity limits.
    ///
    /// **⚠️ Performance Warning:**
    /// This callback executes synchronously while holding the `AgentState` write lock.
    /// Do NOT perform blocking I/O (like writing to a database or file) directly inside this closure.
    /// Instead, extract the required handles (e.g., from `state.extensions()`) and `tokio::spawn`
    /// an asynchronous task to handle the evicted messages.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_evict<F>(mut self, handler: F) -> Self
    where
        F: Fn(&AgentState, Vec<Arc<Message>>) + Send + Sync + 'static,
    {
        self.on_evict_handler = Some(Arc::new(handler));
        self
    }

    /// Registers a callback that triggers whenever conversation history is evicted.
    /// (See native documentation for performance warnings regarding blocking operations).
    #[cfg(target_arch = "wasm32")]
    pub fn on_evict<F>(mut self, handler: F) -> Self
    where
        F: Fn(&AgentState, Vec<Arc<Message>>) + 'static,
    {
        self.on_evict_handler = Some(Arc::new(handler));
        self
    }
}
