use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingEngineTrait: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait]
pub trait MemoryStoreTrait: Send + Sync {
    async fn add(&mut self, text: String, embedding: Option<Vec<f32>>) -> Result<()>;
    async fn search(
        &self,
        query: &str,
        query_embedding: Option<Vec<f32>>,
        limit: usize,
    ) -> Result<Vec<String>>;
}

pub struct MemoryManager {
    embedder: Option<Box<dyn EmbeddingEngineTrait>>,
    store: Box<dyn MemoryStoreTrait>,
    pub top_k: usize,
}

impl MemoryManager {
    pub fn new(store: Box<dyn MemoryStoreTrait>) -> Self {
        Self {
            embedder: None,
            store,
            top_k: 3,
        }
    }

    pub fn with_embedder(mut self, embedder: Box<dyn EmbeddingEngineTrait>) -> Self {
        self.embedder = Some(embedder);
        self
    }

    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    pub async fn add_memory(&mut self, text: String) -> Result<()> {
        let embedding = if let Some(embedder) = &self.embedder {
            Some(embedder.embed(&text).await?)
        } else {
            None
        };
        self.store.add(text, embedding).await
    }

    pub async fn retrieve_memory(&self, query: &str) -> Result<Vec<String>> {
        let embedding = if let Some(embedder) = &self.embedder {
            Some(embedder.embed(query).await?)
        } else {
            None
        };
        self.store.search(query, embedding, self.top_k).await
    }
}
