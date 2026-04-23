// src/llm/providers/openai/mod.rs
pub mod translator;

use crate::error::{AmbiError, Result};
use crate::llm::LLMEngineTrait;
use crate::types::config::OpenAIEngineConfig;
use crate::types::LLMRequest;
use async_openai::config::OpenAIConfig;
use async_openai::Client;
use async_trait::async_trait;
use futures::StreamExt;
use log::debug;
use std::collections::BTreeMap;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct OpenAIEngine {
    client: Client<OpenAIConfig>,
    cfg: OpenAIEngineConfig,
}

impl OpenAIEngine {
    pub fn load(openai_cfg: OpenAIEngineConfig) -> Result<Self> {
        let mut config = OpenAIConfig::new().with_api_key(openai_cfg.api_key.clone());
        config = config.with_api_base(&openai_cfg.base_url);
        let client = Client::with_config(config);

        Ok(Self {
            client,
            cfg: openai_cfg,
        })
    }

    pub async fn generate_response_sync(&self, request: LLMRequest) -> Result<String> {
        let api_request = self.get_request(self.cfg.model_name.clone(), request, false)?;
        let response = self
            .client
            .chat()
            .create(api_request)
            .await
            .map_err(|e| AmbiError::EngineError(e.to_string()))?;

        let choice = response.choices.into_iter().next().ok_or_else(|| {
            AmbiError::EngineError("No choices returned by OpenAI API".to_string())
        })?;

        if let Some(tool_calls) = choice.message.tool_calls {
            let v = serde_json::to_value(&tool_calls).unwrap_or_default();
            let mut simulated = String::new();
            if let Some(arr) = v.as_array() {
                for tc in arr {
                    if let Some(func) = tc.get("function") {
                        let name = func
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or_default();
                        let args = func
                            .get("arguments")
                            .and_then(|a| a.as_str())
                            .unwrap_or_default();
                        simulated.push_str(&format!(
                            "[TOOL_CALL]{{\"name\":\"{}\",\"args\":{}}}[/TOOL_CALL]",
                            name, args
                        ));
                    }
                }
            }
            return Ok(simulated);
        }

        Ok(choice.message.content.unwrap_or_default())
    }

    pub async fn generate_response_stream(
        &self,
        request: LLMRequest,
        tx: Sender<Result<String>>,
    ) -> Result<()> {
        if let Some(msg) = request.history.last() {
            debug!("\n[OpenAI API] Request\n====================\n{}", msg);
        }

        let api_request = self.get_request(self.cfg.model_name.clone(), request, true)?;
        let mut stream = self
            .client
            .chat()
            .create_stream(api_request)
            .await
            .map_err(|e| AmbiError::EngineError(e.to_string()))?;

        let mut tool_calls_map: BTreeMap<u32, (String, String)> = BTreeMap::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for choice in response.choices {
                        if let Some(tool_calls) = choice.delta.tool_calls {
                            let v = serde_json::to_value(&tool_calls).unwrap_or_default();
                            if let Some(arr) = v.as_array() {
                                for tc in arr {
                                    if let Some(idx) = tc.get("index").and_then(|i| i.as_u64()) {
                                        let entry = tool_calls_map.entry(idx as u32).or_default();
                                        if let Some(func) = tc.get("function") {
                                            if let Some(n) =
                                                func.get("name").and_then(|n| n.as_str())
                                            {
                                                entry.0.push_str(n);
                                            }
                                            if let Some(a) =
                                                func.get("arguments").and_then(|a| a.as_str())
                                            {
                                                entry.1.push_str(a);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(content) = choice.delta.content {
                            if tx.send(Ok(content)).await.is_err() {
                                return Ok(());
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(AmbiError::EngineError(format!(
                            "Stream interrupted: {}",
                            e
                        ))))
                        .await;
                    return Err(AmbiError::EngineError(e.to_string()));
                }
            }
        }

        if !tool_calls_map.is_empty() {
            let mut simulated = String::new();
            for (name, args) in tool_calls_map.values() {
                simulated.push_str(&format!(
                    "[TOOL_CALL]{{\"name\":\"{}\",\"args\":{}}}[/TOOL_CALL]",
                    name, args
                ));
            }
            let _ = tx.send(Ok(simulated)).await;
        }
        Ok(())
    }
}

#[async_trait]
impl LLMEngineTrait for OpenAIEngine {
    async fn chat(&mut self, request: LLMRequest) -> Result<String> {
        self.generate_response_sync(request).await
    }
    async fn chat_stream(&mut self, request: LLMRequest, tx: Sender<Result<String>>) {
        if let Err(e) = self.generate_response_stream(request, tx.clone()).await {
            let _ = tx.send(Err(e)).await;
        }
    }
    fn reset_context(&mut self) {}
}
