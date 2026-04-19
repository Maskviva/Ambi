use crate::core::llm::chat_template::{ChatTemplate, ChatTemplateType};
use crate::core::llm::formatter::StreamFormatter;
use crate::core::llm::handler::LLMEngine;
use crate::core::llm::EngineConfig;
use crate::core::tool::{DynTool, Tool, ToolDefinition, ToolManager};

use anyhow::Result;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Message {
    System { content: String },
    User { content: String },
    Tool { target: String, content: String },
    Assistant { target: String, content: String },
}

#[derive(Serialize, Deserialize)]
pub struct CompletionRequest {
    pub chat_history: Vec<Message>,
    pub tools: Vec<ToolDefinition>,

    #[serde(skip)]
    __tool_map: HashMap<String, Box<dyn DynTool>>,
    __requested: bool,
}

pub struct Agent {
    pub completion_request: Arc<TokioMutex<CompletionRequest>>,
    pub llm_engine: Arc<TokioMutex<LLMEngine>>,
    pub system_prompt: String,
    pub template: ChatTemplate,
}

impl Agent {
    pub fn make(engine_cfg: EngineConfig) -> Result<Self> {
        let engine = LLMEngine::load(engine_cfg)?;
        let llm_engine = Arc::new(TokioMutex::new(engine));

        let completion_request = Arc::new(TokioMutex::new(CompletionRequest {
            chat_history: Vec::new(),
            tools: Vec::new(),
            __tool_map: HashMap::new(),
            __requested: false,
        }));

        Ok(Self {
            llm_engine,
            completion_request,
            system_prompt: String::new(),
            template: ChatTemplateType::Chatml.as_template(),
        })
    }

    pub async fn chat(&mut self, prompt: &str) -> String {
        self.completion_request
            .lock()
            .await
            .chat_history
            .push(Message::User {
                content: prompt.to_string(),
            });

        let mut target = prompt.to_string();
        let mut is_tool_call = false;
        let mut final_formatted_output = String::new();

        loop {
            let input_prompt = Self::get_prompt(
                &self.completion_request,
                &self.system_prompt,
                &self.template,
            )
            .await;

            let res = self
                .llm_engine
                .lock()
                .await
                .chat(&input_prompt, is_tool_call)
                .await
                .unwrap_or_default();

            {
                let mut req = self.completion_request.lock().await;
                req.chat_history.push(Message::Assistant {
                    target: target.clone(),
                    content: res.clone(),
                });
                req.__requested = true;
            }

            let mut formatter = StreamFormatter::new();
            let cleaned_res = formatter.push(&res) + &formatter.flush();
            final_formatted_output.push_str(&cleaned_res);

            match Self::handle_tool_call(&self.completion_request, &res).await {
                Some((name, args, tool_msg)) => {
                    final_formatted_output.push_str(&format!(
                        "\n\n[TOOL_CALL]: {}({})\n[TOOL]: {}\n\n",
                        name, args, tool_msg
                    ));
                    target = tool_msg;
                    is_tool_call = true;
                }
                None => return final_formatted_output.trim().to_string(),
            }
        }
    }

    pub async fn chat_stream(
        &mut self,
        prompt: &str,
    ) -> Result<Pin<Box<UnboundedReceiverStream<Result<String, String>>>>, ()> {
        use tokio::sync::mpsc::unbounded_channel;

        let llm_engine = Arc::clone(&self.llm_engine);
        let completion_request = Arc::clone(&self.completion_request);
        let system_prompt = self.system_prompt.clone();
        let prompt = prompt.to_string();

        completion_request
            .lock()
            .await
            .chat_history
            .push(Message::User {
                content: prompt.clone(),
            });

        let (tx_out, rx_out) = unbounded_channel::<Result<String, String>>();
        let template_clone = self.template.clone();

        tokio::spawn(async move {
            let mut target = prompt;
            let mut is_tool_call = false;

            loop {
                let input_prompt =
                    Self::get_prompt(&completion_request, &system_prompt, &template_clone).await;

                let (tx_llm, mut rx_llm) = unbounded_channel::<String>();
                let llm_engine_clone = Arc::clone(&llm_engine);

                let llm_task = tokio::spawn(async move {
                    let mut engine = llm_engine_clone.lock().await;
                    engine
                        .chat_stream(&input_prompt, is_tool_call, tx_llm)
                        .await;
                });

                let mut full_output = String::with_capacity(1024);
                let mut formatter = StreamFormatter::new();

                while let Some(token) = rx_llm.recv().await {
                    full_output.push_str(&token);

                    let cleaned_text = formatter.push(&token);
                    if !cleaned_text.is_empty() {
                        let _ = tx_out.send(Ok(cleaned_text));
                    }
                }

                let flushed = formatter.flush();
                if !flushed.is_empty() {
                    let _ = tx_out.send(Ok(flushed));
                }

                let _ = llm_task.await;

                {
                    let mut req = completion_request.lock().await;
                    req.chat_history.push(Message::Assistant {
                        target: target.clone(),
                        content: full_output.clone(),
                    });
                    req.__requested = true;
                }

                match Self::handle_tool_call(&completion_request, &full_output).await {
                    Some((name, args, tool_msg)) => {
                        let formatted_tool_block = format!(
                            "\n\n[TOOL_CALL]: {}({})\n[TOOL]: {}\n\n",
                            name, args, tool_msg
                        );
                        let _ = tx_out.send(Ok(formatted_tool_block));

                        target = tool_msg;
                        is_tool_call = true;
                    }
                    None => break,
                }
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx_out)))
    }

    pub async fn clear_history(&self) -> Vec<Message> {
        self.completion_request.lock().await.chat_history.clear();
        self.completion_request.lock().await.chat_history.clone()
    }

    pub fn preamble(mut self, system_prompt: &str) -> Self {
        self.system_prompt = system_prompt.to_string();
        self
    }

    pub fn template(mut self, template_type: ChatTemplateType) -> Self {
        self.template = template_type.as_template();
        self
    }

    pub fn tool<T: Tool + 'static>(self, tool: T) -> Result<Self> {
        let def = tool.definition();

        {
            let mut req = self.completion_request.try_lock()?;

            if !req.tools.iter().any(|t| t.name == def.name) {
                req.tools.push(ToolDefinition {
                    name: def.name.clone(),
                    description: def.description,
                    parameters: def.parameters,
                });

                req.__tool_map.insert(def.name, Box::new(tool));
            }
        }

        Ok(self)
    }

    async fn handle_tool_call(
        req_mutex: &TokioMutex<CompletionRequest>,
        assistant_response: &str,
    ) -> Option<(String, String, String)> {
        let (name, args) = ToolManager::parse_tool_call(assistant_response)?;

        let mut req = req_mutex.lock().await;
        let tool_result = ToolManager::run_tool(&req.__tool_map, name.clone(), &args).await;

        let tool_msg = tool_result.unwrap_or_else(|e| {
            error!("Failed to execute tool '{}': {}", name, e);
            format!("Failed to execute tool '{}': {}", name, e)
        });

        req.chat_history.push(Message::Tool {
            target: assistant_response.to_string(),
            content: tool_msg.clone(),
        });

        Some((name, args.to_string(), tool_msg))
    }

    async fn get_prompt(
        req_mutex: &TokioMutex<CompletionRequest>,
        system_prompt: &str,
        tpl: &ChatTemplate,
    ) -> String {
        let req = req_mutex.lock().await;
        Self::build_prompt(system_prompt, &req, tpl)
    }

    fn build_prompt(system_prompt: &str, req: &CompletionRequest, tpl: &ChatTemplate) -> String {
        let mut prompt = String::with_capacity(2048);

        let _ = write!(
            prompt,
            "{}{}{}",
            tpl.system_prefix, system_prompt, tpl.system_suffix
        );

        let tool_content = ToolManager::tool_prompt(req.tools.clone());
        if !tool_content.is_empty() {
            let _ = write!(
                prompt,
                "{}{}{}",
                tpl.system_prefix, tool_content, tpl.system_suffix
            );
        }

        for msg in &req.chat_history {
            match msg {
                Message::System { content } => {
                    let _ = write!(
                        prompt,
                        "{}{}{}",
                        tpl.system_prefix, content, tpl.system_suffix
                    );
                }
                Message::User { content } => {
                    let _ = write!(prompt, "{}{}{}", tpl.user_prefix, content, tpl.user_suffix);
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
