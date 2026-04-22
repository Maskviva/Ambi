use super::{Agent, CompletionRequest};
use crate::agent::core::history::ChatHistory;
use crate::agent::tool::{DefaultToolParser, Tool, ToolCallParser, ToolDefinition};
use crate::llm::{ChatTemplateType, LLMEngine, LLMEngineConfig, LLMEngineTrait};
use crate::types::message::Message;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

impl Agent {
    pub async fn make(engine_cfg: LLMEngineConfig) -> Result<Self> {
        let engine = tokio::task::spawn_blocking(move || LLMEngine::load(engine_cfg))
            .await
            .map_err(|e| anyhow!("Failed to spawn blocking task: {}", e))??;

        Ok(Self::init_agent(engine))
    }

    pub fn with_custom_engine(custom_backend: Box<dyn LLMEngineTrait>) -> Result<Self> {
        let engine = LLMEngine::from_custom(custom_backend);
        Ok(Self::init_agent(engine))
    }

    pub(super) fn init_agent(engine: LLMEngine) -> Self {
        let llm_engine = Arc::new(TokioMutex::new(engine));
        let completion_request = Arc::new(TokioMutex::new(CompletionRequest {
            chat_history: ChatHistory::new(),
            __requested: false,
        }));

        Self {
            llm_engine,
            completion_request,
            system_prompt: String::new(),
            template: ChatTemplateType::Chatml.as_template(),
            tools_def: Arc::new(Vec::new()),
            tool_map: Arc::new(HashMap::new()),
            tool_parser: Arc::new(DefaultToolParser::make()),
            on_evict_handler: None,
            max_iterations: 10,
            enable_formatting: false,
            eviction_strategy: (2, 6),
        }
    }

    pub fn enable_formatting(mut self, enable: bool) -> Self {
        self.enable_formatting = enable;
        self
    }

    pub fn with_eviction_strategy(mut self, keep_head: usize, keep_tail: usize) -> Self {
        self.eviction_strategy = (keep_head, keep_tail);
        self
    }

    pub fn preamble(mut self, system_prompt: &str) -> Self {
        self.system_prompt = system_prompt.to_string();
        self
    }

    pub fn template(mut self, template_type: ChatTemplateType) -> Self {
        self.template = template_type.as_template();
        self
    }

    pub fn tool<T: Tool + 'static>(mut self, tool: T) -> Result<Self> {
        let def = tool.definition();
        let mut defs = Arc::try_unwrap(self.tools_def).unwrap_or_else(|arc| (*arc).clone());
        let mut map = Arc::try_unwrap(self.tool_map).unwrap_or_else(|arc| (*arc).clone());

        if !defs.iter().any(|t| t.name == def.name) {
            defs.push(ToolDefinition {
                name: def.name.clone(),
                description: def.description,
                parameters: def.parameters,
                timeout_secs: def.timeout_secs,
                max_retries: def.max_retries,
                is_idempotent: def.is_idempotent,
            });
            map.insert(def.name, Arc::new(tool));
        }

        self.tools_def = Arc::new(defs);
        self.tool_map = Arc::new(map);
        Ok(self)
    }

    pub fn with_tool_parser<P: ToolCallParser + 'static>(mut self, parser: P) -> Self {
        self.tool_parser = Arc::new(parser);
        self
    }

    pub fn on_evict<F>(mut self, handler: F) -> Self
    where
        F: Fn(Vec<Arc<Message>>) + Send + Sync + 'static,
    {
        self.on_evict_handler = Some(Arc::new(handler));
        self
    }
}
