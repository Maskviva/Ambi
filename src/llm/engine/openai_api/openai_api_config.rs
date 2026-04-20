use serde::Deserialize;

#[cfg(feature = "openai-api")]
#[derive(Debug, Deserialize, Clone)]
pub struct OpenAIEngineConfig {
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
    pub temp: f32,
    pub top_p: f32,
}

#[cfg(feature = "openai-api")]
impl OpenAIEngineConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.api_key.trim().is_empty() {
            return Err(anyhow::anyhow!("OpenAI API Key cannot be empty"));
        }
        if self.temp < 0.0 || self.temp > 2.0 {
            return Err(anyhow::anyhow!("Temperature must be between 0.0 and 2.0"));
        }
        Ok(())
    }
}
