use crate::agent::message::ContentPart;
use crate::agent::Message;
use crate::llm::engine::openai_api::openai_api_config::OpenAIEngineConfig;
use crate::llm::handler::LLMRequest;
use anyhow::Result;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use futures::StreamExt;
use log::{debug, error};
use tokio::sync::mpsc::Sender;

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

        if let Some(Message::User { content }) = request.history.last() {
            for part in content {
                if let ContentPart::Text { text } = part {
                    new_prompt.push_str(text);
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
                    let _ = tx
                        .send(Err(anyhow::anyhow!("Stream interrupted: {}", e)))
                        .await;
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

        if !request.memory_context.is_empty() {
            messages.push(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(request.memory_context)
                    .build()?
                    .into(),
            );
        }

        for msg in request.history {
            match msg {
                Message::System { content } => {
                    messages.push(
                        ChatCompletionRequestSystemMessageArgs::default()
                            .content(content)
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
                            .content(content)
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
