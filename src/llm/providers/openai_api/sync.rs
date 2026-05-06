// src/llm/providers/openai_api/sync.rs

use super::OpenAIEngine;
use crate::error::{AmbiError, Result};
use crate::types::LLMRequest;

impl OpenAIEngine {
    /// Generates a synchronous response from the OpenAI API.
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
}
