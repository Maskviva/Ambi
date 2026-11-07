// src/llm/providers/openai_api/sync.rs

//! Synchronous (non-streaming) response generation for OpenAI-compatible APIs.
//!
//! Translates native OpenAI tool call responses into Ambi's tag-based format.

use super::OpenAIEngine;
use crate::error::{AmbiError, Result};
use crate::types::LLMRequest;

impl OpenAIEngine {
    /// Generates a synchronous response from the OpenAI API.
    pub async fn generate_response_sync(&self, request: LLMRequest) -> Result<String> {
        let tool_tags = request.tool_tags.clone();
        let api_request = self.get_request(self.cfg.model_name.clone(), request, false)?;

        let response = self.client.chat().create(api_request).await.map_err(|e| {
            AmbiError::engine_kind(crate::error::EngineErrorKind::Transport, e.to_string())
        })?;

        let choice = response.choices.into_iter().next().ok_or_else(|| {
            AmbiError::engine_kind(
                crate::error::EngineErrorKind::Transport,
                "No choices returned by OpenAI API".to_string(),
            )
        })?;

        if let Some(tool_calls) = choice.message.tool_calls {
            let (start_tag, end_tag) = &tool_tags;
            let simulated: String = tool_calls
                .iter()
                .filter_map(|tc| super::translator::render_tool_call(tc, start_tag, end_tag))
                .collect();
            return Ok(simulated);
        }

        Ok(choice.message.content.unwrap_or_default())
    }
}
