// src/llm/providers/openai/mod.rs
pub mod translator;

use crate::error::{AmbiError, Result};
use crate::llm::LLMEngineTrait;
use crate::types::config::OpenAIEngineConfig;
use crate::types::LLMRequest;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::ChatCompletionMessageToolCallChunk;
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
        let tool_tags = request.tool_tags.clone();
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
                        let (start_tag, end_tag) = &tool_tags;

                        simulated.push_str(&format!(
                            "{}{{\"name\":\"{}\",\"args\":{}}}{}",
                            start_tag, name, args, end_tag
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

        let tool_tags = request.tool_tags.clone();
        let api_request = self.get_request(self.cfg.model_name.clone(), request, true)?;
        let mut stream = self
            .client
            .chat()
            .create_stream(api_request)
            .await
            .map_err(|e| AmbiError::EngineError(e.to_string()))?;

        let mut tool_calls_map: BTreeMap<u32, (String, String)> = BTreeMap::new();
        let mut tool_calls_started = false;

        while let Some(result) = stream.next().await {
            let response = match result {
                Ok(resp) => resp,
                Err(e) => {
                    let _ = tx
                        .send(Err(AmbiError::EngineError(format!(
                            "Stream interrupted: {}",
                            e
                        ))))
                        .await;
                    return Err(AmbiError::EngineError(e.to_string()));
                }
            };

            for choice in response.choices {
                if let Some(tool_calls) = choice.delta.tool_calls {
                    tool_calls_started = true;
                    Self::collect_tool_call_delta(&mut tool_calls_map, tool_calls);
                }

                if !tool_calls_started {
                    if let Some(content) = choice.delta.content {
                        if tx.send(Ok(content)).await.is_err() {
                            return Ok(());
                        }
                    }
                }
            }
        }

        if !tool_calls_map.is_empty() {
            let simulated = tool_calls_map
                .values()
                .map(|(name, args)| {
                    format!(
                        "{}{{\"name\":\"{}\",\"args\":{}}}{}",
                        tool_tags.0, name, args, tool_tags.1
                    )
                })
                .collect::<Vec<_>>()
                .join("");
            let _ = tx.send(Ok(simulated)).await;
        }
        Ok(())
    }

    fn collect_tool_call_delta(
        map: &mut BTreeMap<u32, (String, String)>,
        calls: Vec<ChatCompletionMessageToolCallChunk>,
    ) {
        let v = serde_json::to_value(&calls).unwrap_or_default();
        let arr = match v.as_array() {
            Some(a) => a,
            None => return,
        };

        for tc in arr {
            let idx = match tc.get("index").and_then(|i| i.as_u64()) {
                Some(i) => i as u32,
                None => continue,
            };
            let func = match tc.get("function") {
                Some(f) => f,
                None => continue,
            };

            let entry = map.entry(idx).or_default();
            if let Some(n) = func.get("name").and_then(|v| v.as_str()) {
                entry.0.push_str(n);
            }
            if let Some(a) = func.get("arguments").and_then(|v| v.as_str()) {
                entry.1.push_str(a);
            }
        }
    }
}

#[async_trait]
impl LLMEngineTrait for OpenAIEngine {
    async fn chat(&self, request: LLMRequest) -> Result<String> {
        self.generate_response_sync(request).await
    }
    async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>) {
        if let Err(e) = self.generate_response_stream(request, tx.clone()).await {
            let _ = tx.send(Err(e)).await;
        }
    }

    fn reset_context(&self) {}
    fn supports_multimodal(&self) -> bool {
        true
    }
}
