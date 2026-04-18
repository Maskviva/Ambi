use anyhow::Result;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use futures::StreamExt;
use log::{debug, error};
use tokio::sync::mpsc::UnboundedSender;

use crate::core::llm::OpenAIEngineConfig;

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
        new_prompt: &str,
        tx: UnboundedSender<String>,
    ) -> Result<()> {
        debug!(
            "\n [OpenAI API] Request \n ========================================\n{}",
            new_prompt
        );

        let model_name = self.cfg.model_name.clone();

        let request = self.get_request(model_name, new_prompt, true)?;

        let mut stream = self.client.chat().create_stream(request).await?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for choice in response.choices {
                        if let Some(content) = choice.delta.content {
                            if tx.send(content).is_err() {
                                
                                    debug!("输出通道已关闭，终止 OpenAI 网络流接收");
                                return Ok(());
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("OpenAI Stream Error: {}", e);
                }
            }
        }
        Ok(())
    }

    pub async fn generate_response_sync(&self, new_prompt: &str) -> Result<String> {
        let model_name = self.cfg.model_name.clone();

        let request = self.get_request(model_name, new_prompt, false)?;

        let response = self.client.chat().create(request).await?;

        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default();

        Ok(content)
    }

    pub fn reset_context(&mut self) {
        // OpenAI 的无状态 API 不需要清理上下文
    }

    fn get_request(
        &self,
        model_name: String,
        new_prompt: &str,
        stream: bool,
    ) -> Result<async_openai::types::chat::CreateChatCompletionRequest> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(model_name)
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(new_prompt)
                .build()?
                .into()])
            .temperature(self.cfg.temp)
            .top_p(self.cfg.top_p)
            .stream(stream)
            .build()?;

        Ok(request)
    }
}
