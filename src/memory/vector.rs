use crate::memory::handler::MemoryStoreTrait;
use anyhow::{anyhow, Result};
use async_trait::async_trait;

struct MemoryItem {
    text: String,
    embedding: Vec<f32>,
}

#[derive(Default)]
pub struct VectorMemoryStore {
    items: Vec<MemoryItem>,
}

impl VectorMemoryStore {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot_product / (norm_a * norm_b)
    }
}

#[async_trait]
impl MemoryStoreTrait for VectorMemoryStore {
    async fn add(&mut self, text: String, embedding: Option<Vec<f32>>) -> Result<()> {
        let emb = embedding.ok_or_else(|| anyhow!("Vector store requires an embedding engine"))?;
        self.items.push(MemoryItem { text, embedding: emb });
        Ok(())
    }

    async fn search(&self, _query: &str, query_embedding: Option<Vec<f32>>, limit: usize) -> Result<Vec<String>> {
        let q_emb = query_embedding.ok_or_else(|| anyhow!("Vector store requires a query embedding"))?;

        let mut scored_items: Vec<(f32, &String)> = self.items.iter()
            .map(|item| {
                let score = Self::cosine_similarity(&q_emb, &item.embedding);
                (score, &item.text)
            })
            .collect();

        scored_items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let result = scored_items.into_iter()
            .take(limit)
            .map(|(_, text)| text.clone())
            .collect();

        Ok(result)
    }
}