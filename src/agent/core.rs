use crate::agent::formatter::StreamFormatter;
use crate::agent::history::ChatHistory;
use crate::agent::message::Message;
use crate::agent::tool::{DynTool, Tool, ToolDefinition, ToolManager};
use crate::llm::chat_template::{ChatTemplate, ChatTemplateType};
use crate::llm::handler::LLMRequest;
use crate::llm::handler::{LLMEngine, LLMEngineTrait};
use crate::llm::LLMEngineConfig;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::wrappers::ReceiverStream;

static MAX_ITERATIONS: usize = 10;

#[derive(Serialize, Deserialize)]
pub struct CompletionRequest {
    pub chat_history: ChatHistory,
    __requested: bool,
}

pub struct Agent {
    pub completion_request: Arc<TokioMutex<CompletionRequest>>,
    pub llm_engine: Arc<TokioMutex<LLMEngine>>,
    pub system_prompt: String,
    pub template: ChatTemplate,

    pub tools_def: Arc<Vec<ToolDefinition>>,
    pub tool_map: Arc<HashMap<String, Arc<dyn DynTool>>>,
}

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

    fn init_agent(engine: LLMEngine) -> Self {
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
        }
    }

    pub async fn chat(&mut self, prompt: &str) -> Result<String> {
        Self::append_user_message(&self.completion_request, prompt).await;

        let mut target = prompt.to_string();
        let mut final_formatted_output = String::new();
        let mut iteration_count = 0;

        loop {
            if iteration_count >= MAX_ITERATIONS {
                return Err(anyhow!("Agent has reached the maximum number of tool call loops ({}), forcibly terminating.", MAX_ITERATIONS));
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
            )
            .await;

            let mut formatter = StreamFormatter::new();
            final_formatted_output.push_str(&formatter.push(&res));
            final_formatted_output.push_str(&formatter.flush());

            let tool_calls =
                Self::handle_tool_calls(&self.completion_request, &self.tool_map, &res).await?;

            if tool_calls.is_empty() {
                return Ok(final_formatted_output.trim().to_string());
            }

            target = Self::process_tool_calls_output(&tool_calls, &mut final_formatted_output);
            iteration_count += 1;
        }
    }

    pub async fn chat_stream(
        &mut self,
        prompt: &str,
    ) -> Result<Pin<Box<ReceiverStream<Result<String, String>>>>, ()> {
        use tokio::sync::mpsc::channel;

        let llm_engine = Arc::clone(&self.llm_engine);
        let completion_request = Arc::clone(&self.completion_request);
        let system_prompt = self.system_prompt.clone();

        Self::append_user_message(&completion_request, prompt).await;

        let prompt_clone = prompt.to_string();
        let (tx_out, rx_out) = channel::<Result<String, String>>(1024);
        let template_clone = self.template.clone();
        let tools_def_clone = Arc::clone(&self.tools_def);
        let tool_map_clone = Arc::clone(&self.tool_map);

        tokio::spawn(async move {
            let mut target = prompt_clone;
            let mut iteration_count = 0;

            loop {
                if iteration_count >= MAX_ITERATIONS {
                    let _ = tx_out
                        .send(Err(format!(
                            "Agent has reached the maximum number of tool call loops ({}), forcibly terminating.",
                            MAX_ITERATIONS
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
                )
                .await;

                let tool_calls = match Self::handle_tool_calls(
                    &completion_request,
                    &tool_map_clone,
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
    ) {
        let mut req = req_mutex.lock().await;
        req.chat_history
            .push(Message::Assistant { target, content });
        req.__requested = true;

        let evicted_msgs = req.chat_history.evict_for_memory(2, 6);
        if !evicted_msgs.is_empty() {
            log::debug!(
                "上下文截断：抽离了 {} 条中间对话放入记忆库",
                evicted_msgs.len()
            );
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

    pub async fn clear_history(&self) {
        self.completion_request.lock().await.chat_history.clear();
        self.llm_engine.lock().await.reset_context();
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
            });
            map.insert(def.name, Arc::new(tool));
        }

        self.tools_def = Arc::new(defs);
        self.tool_map = Arc::new(map);
        Ok(self)
    }

    async fn handle_tool_calls(
        req_mutex: &TokioMutex<CompletionRequest>,
        tool_map: &HashMap<String, Arc<dyn DynTool>>,
        assistant_response: &str,
    ) -> Result<Vec<(String, String, String)>> {
        let calls = ToolManager::parse_tool_calls(assistant_response);
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

    async fn get_llm_request(
        req_mutex: &TokioMutex<CompletionRequest>,
        system_prompt: &str,
        tpl: &ChatTemplate,
        tools: &[ToolDefinition],
    ) -> LLMRequest {
        let req = req_mutex.lock().await;
        let formatted_prompt = Self::build_prompt(system_prompt, &req, tpl, tools);
        let tool_prompt = ToolManager::tool_prompt(tools.to_vec());

        LLMRequest {
            system_prompt: system_prompt.to_string(),
            history: req.chat_history.all().to_vec(),
            formatted_prompt,
            tool_prompt,
        }
    }

    fn build_prompt(
        system_prompt: &str,
        req: &CompletionRequest,
        tpl: &ChatTemplate,
        tools: &[ToolDefinition],
    ) -> String {
        let mut prompt = String::with_capacity(2048);
        let _ = write!(
            prompt,
            "{}{}{}",
            tpl.system_prefix, system_prompt, tpl.system_suffix
        );

        let tool_content = ToolManager::tool_prompt(tools.to_vec());
        if !tool_content.is_empty() {
            let _ = write!(
                prompt,
                "{}{}{}",
                tpl.system_prefix, tool_content, tpl.system_suffix
            );
        }

        for msg in req.chat_history.all() {
            match msg {
                Message::System { content } => {
                    let _ = write!(
                        prompt,
                        "{}{}{}",
                        tpl.system_prefix, content, tpl.system_suffix
                    );
                }
                Message::User { .. } => {
                    let text = msg.get_text_content();
                    let _ = write!(prompt, "{}{}{}", tpl.user_prefix, text, tpl.user_suffix);
                }
                Message::Tool { content, .. } => {
                    let _ = write!(prompt, "{}{}{}", tpl.tool_prefix, content, tpl.tool_suffix);
                }
                Message::Assistant { content, .. } => {
                    let _ = write!(
                        prompt,
                        "{}{}{}",
                        tpl.assistant_prefix, content, tpl.assistant_suffix
                    );
                }
            }
        }
        prompt.push_str(&tpl.assistant_prefix);
        prompt
    }
}
