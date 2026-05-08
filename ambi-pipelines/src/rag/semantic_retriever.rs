//! A `Retriever` implementation backed by `ambi-memory`'s `SemanticMemoryProvider`.

use super::document::Document;
use super::retriever::Retriever;
use ambi_memory::SemanticMemoryProvider;
use async_trait::async_trait;
use std::sync::Arc;

/// A `Retriever` that wraps `ambi-memory`'s `SemanticMemoryProvider` to serve stored memories as documents.
pub struct SemanticMemoryRetriever<M: SemanticMemoryProvider> {
    memory_provider: Arc<M>,
    session_id: String,
    limit: usize,
}

impl<M: SemanticMemoryProvider> SemanticMemoryRetriever<M> {
    /// Creates a new `SemanticMemoryRetriever` that searches the given session's memories.
    pub fn new(memory_provider: M, session_id: impl Into<String>, limit: usize) -> Self {
        Self {
            memory_provider: Arc::new(memory_provider),
            session_id: session_id.into(),
            limit,
        }
    }
}

#[async_trait]
impl<M: SemanticMemoryProvider + 'static> Retriever for SemanticMemoryRetriever<M> {
    async fn retrieve(
        &self,
        query: &str,
    ) -> Result<Vec<Document>, Box<dyn std::error::Error + Send + Sync>> {
        let memories = self
            .memory_provider
            .search_memories(&self.session_id, query, self.limit)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let docs = memories
            .into_iter()
            .enumerate()
            .map(|(i, text)| Document::new(format!("memory_chunk_{}", i), text, 1.0))
            .collect();

        Ok(docs)
    }
}
