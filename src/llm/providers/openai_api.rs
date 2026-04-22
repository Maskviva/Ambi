use crate::llm::{LLMEngineTrait, LLMRequest};
use crate::types::message::{ContentPart, Message};
use anyhow::{anyhow, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use async_trait::async_trait;
use futures::StreamExt;
use log::{debug, error};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAIEngineConfig {
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
    pub temp: f32,
    pub top_p: f32,
}

impl OpenAIEngineConfig {
    pub fn validate(&self) -> Result<()> {
        if self.api_key.trim().is_empty() {
            return Err(anyhow!("OpenAI API Key cannot be empty"));
        }
        if self.temp < 0.0 || self.temp > 2.0 {
            return Err(anyhow!("Temperature must be between 0.0 and 2.0"));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct OpenAIEngine {
    client: Client<OpenAIConfig>,
    cfg: OpenAIEngineConfig,
}

impl OpenAIEngine {
    pub fn load(openai_cfg: OpenAIEngineConfig) -> Result<Self> {
        let api_key = openai_cfg.api_key.clone();
        let mut config = OpenAIConfig::new().with_api_key(api_key);

        config = config.with_api_base(&openai_cfg.base_url);

        let client = Client::with_config(config);

        Ok(Self {
            client,
            cfg: openai_cfg,
        })
    }

    pub async fn generate_response_stream(
        &self,
        request: LLMRequest,
        tx: Sender<Result<String, anyhow::Error>>,
    ) -> Result<()> {
        let mut new_prompt = String::new();

        if let Some(msg) = request.history.last() {
            if let Message::User { content } = &**msg {
                for part in content {
                    if let ContentPart::Text { text } = part {
                        new_prompt.push_str(text);
                    }
                }
            }
        }

        debug!(
            "\n[OpenAI API] Request\n========================================\n{}",
            new_prompt
        );

        let model_name = self.cfg.model_name.clone();

        let api_request = self.get_request(model_name, request, true)?;

        let mut stream = self.client.chat().create_stream(api_request).await?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for choice in response.choices {
                        if let Some(content) = choice.delta.content {
                            if tx.send(Ok(content)).await.is_err() {
                                debug!("Output channel closed, terminating OpenAI stream.");
                                return Ok(());
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("OpenAI Stream Error: {}", e);
                    let _ = tx.send(Err(anyhow!("Stream interrupted: {}", e))).await;
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }

    pub async fn generate_response_sync(&self, request: LLMRequest) -> Result<String> {
        let model_name = self.cfg.model_name.clone();

        let api_request = self.get_request(model_name, request, false)?;

        let response = self.client.chat().create(api_request).await?;

        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default();

        Ok(content)
    }

    pub fn reset_context(&mut self) {}

    fn get_request(
        &self,
        model_name: String,
        request: LLMRequest,
        stream: bool,
    ) -> Result<CreateChatCompletionRequest> {
        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        let mut sys_content = request.system_prompt;
        if !request.tool_prompt.is_empty() {
            if !sys_content.is_empty() {
                sys_content.push_str("\n\n");
            }
            sys_content.push_str(&request.tool_prompt);
        }

        if !sys_content.is_empty() {
            messages.push(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(sys_content)
                    .build()?
                    .into(),
            );
        }

        for msg in request.history {
            match &*msg {
                Message::System { content } => {
                    messages.push(
                        ChatCompletionRequestSystemMessageArgs::default()
                            .content(content.clone())
                            .build()?
                            .into(),
                    );
                }
                Message::User { .. } => {
                    messages.push(
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(msg.get_text_content())
                            .build()?
                            .into(),
                    );
                }
                Message::Assistant { content, .. } => {
                    messages.push(
                        ChatCompletionRequestAssistantMessageArgs::default()
                            .content(content.clone())
                            .build()?
                            .into(),
                    );
                }
                Message::Tool { content, .. } => {
                    let tool_result = format!(
                        "\n---[TOOL EXECUTION RESULT START]---\n{}\n---[TOOL EXECUTION RESULT END]---\n",
                        content
                    );

                    let mut user_msg_args = ChatCompletionRequestUserMessageArgs::default();
                    user_msg_args.content(tool_result);
                    user_msg_args.name("tool_runtime");
                    messages.push(user_msg_args.build()?.into());
                }
            }
        }

        let request = CreateChatCompletionRequestArgs::default()
            .model(model_name)
            .messages(messages)
            .temperature(self.cfg.temp)
            .top_p(self.cfg.top_p)
            .stream(stream)
            .build()?;

        Ok(request)
    }
}

#[async_trait]
impl LLMEngineTrait for OpenAIEngine {
    async fn chat(&mut self, request: LLMRequest) -> Result<String> {
        self.generate_response_sync(request).await.map_err(|e| {
            error!("OpenAI model generation error: {}", e);
            anyhow!("OpenAI error: {}", e)
        })
    }

    async fn chat_stream(
        &mut self,
        request: LLMRequest,
        tx: Sender<Result<String, anyhow::Error>>,
    ) {
        if let Err(e) = self.generate_response_stream(request, tx.clone()).await {
            error!("OpenAI stream generation error: {}", e);
            let _ = tx.send(Err(anyhow!("OpenAI API Error: {}", e))).await;
        }
    }

    fn reset_context(&mut self) {
        self.reset_context();
    }
}
