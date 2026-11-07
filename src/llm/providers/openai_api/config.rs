// src/llm/providers/openai_api/config.rs

//! Configuration properties for network-based API engines.

use crate::error::AmbiError;
use serde::Deserialize;

/// Configuration settings for OpenAI-compatible cloud APIs.
///
/// This configuration is used to connect the Agent to remote endpoints that follow
/// the standard OpenAI Chat Completions API format. It can be used for OpenAI,
/// DeepSeek, Groq, or even local proxy servers like Ollama or vLLM.
///
/// # Examples
///
/// ```rust
/// use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
///
/// // Using the convenience builder pattern:
/// let config = OpenAIEngineConfig::create("your-api-key".to_string(), "gpt-4o")
///     .temp(0.7)
///     .top_p(0.95);
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct OpenAIEngineConfig {
    /// The secret authorization token for the endpoint.
    pub api_key: String,
    /// The base URL (e.g., `https://api.openai.com/v1`).
    pub base_url: String,
    /// The explicit model tag (e.g., `gpt-4o`).
    pub model_name: String,
    /// The sampling temperature.
    pub temp: f32,
    /// The top-p (nucleus) sampling threshold.
    pub top_p: f32,
    /// Whether the endpoint accepts image input.
    ///
    /// The protocol says nothing about vision, so this is off unless the caller sets it or names a
    /// model known to accept images. When it is off, the framework rejects image input up front
    /// instead of forwarding it to an endpoint that will fail it. Defaults to off when read from a
    /// config file that predates the field.
    #[serde(default)]
    pub supports_vision: bool,
}

impl OpenAIEngineConfig {
    /// Validates the API parameters before executing network requests.
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.api_key.trim().is_empty() {
            return Err(AmbiError::engine_kind(
                crate::error::EngineErrorKind::InvalidRequest,
                "OpenAI API Key cannot be empty".to_string(),
            ));
        }
        if self.temp < 0.0 || self.temp > 2.0 {
            return Err(AmbiError::engine_kind(
                crate::error::EngineErrorKind::InvalidRequest,
                "Temperature must be between 0.0 and 2.0".to_string(),
            ));
        }
        Ok(())
    }

    /// A convenience constructor to quickly initialize the configuration with required fields.
    ///
    /// **Defaults:**
    /// - `base_url`: "https://api.openai.com/v1"
    /// - `temp`: 0.0 (greedy; the most likely token is always chosen)
    /// - `top_p`: 1.0 (nucleus sampling disabled, so it does not fight `temp`)
    pub fn create(api_key: String, model_name: &str) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model_name: model_name.to_string(),
            temp: 0.0,
            top_p: 1.0,
            supports_vision: model_supports_vision(model_name),
        }
    }

    /// States whether this endpoint accepts image input.
    ///
    /// Needed for compatible endpoints whose model name is not one this crate recognises.
    pub fn with_vision(mut self, supports_vision: bool) -> Self {
        self.supports_vision = supports_vision;
        self
    }

    /// Builder method to override the default API base URL.
    /// Highly useful for connecting to local engines (e.g., vLLM) or alternative proxy endpoints.
    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// Builder method to set the sampling temperature (0.0 to 2.0).
    pub fn temp(mut self, temp: f32) -> Self {
        self.temp = temp;
        self
    }

    /// Builder method to set the nucleus sampling threshold.
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }
}

/// Whether a model name is one known to accept image input.
///
/// Matched on a prefix so dated snapshots (`gpt-4o-2024-08-06`) are covered without listing each
/// one. Anything unrecognised is treated as text-only; callers with a vision-capable endpoint under
/// another name opt in with [`OpenAIEngineConfig::with_vision`].
fn model_supports_vision(model_name: &str) -> bool {
    const VISION_PREFIXES: [&str; 5] = ["gpt-4o", "gpt-4-turbo", "gpt-4-vision", "o1", "o3"];
    let name = model_name.to_ascii_lowercase();
    VISION_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}
