use crate::memory::handler::EmbeddingEngineTrait;
use anyhow::{anyhow, Result};
use async_openai::types::embeddings::CreateEmbeddingRequestArgs;
use async_openai::{config::OpenAIConfig, Client};
use async_trait::async_trait;

pub struct OpenAIEmbeddingEngine {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAIEmbeddingEngine {
    pub fn new(api_key: String, base_url: String, model: &str) -> Self {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);
        Self {
            client: Client::with_config(config),
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl EmbeddingEngineTrait for OpenAIEmbeddingEngine {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input([text])
            .build()?;

        let response = self.client.embeddings().create(request).await?;
        let embedding_data = response
            .data
            .first()
            .ok_or_else(|| anyhow!("No embedding data returned"))?;

        Ok(embedding_data.embedding.clone())
    }
}
