//! Vector semantic memory provider for long-term conversational recall.
//!
//! Implementations should pair an embedding service (e.g. OpenAI) with a
//! vector database (e.g. Milvus, Qdrant) to enable similarity-based retrieval.

use crate::error::Result;
use async_trait::async_trait;

/// Semantic (vector) memory interface for storing and searching past experiences.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait SemanticMemoryProvider: Send + Sync {
    /// Embed and persist a piece of content for the given session.
    async fn add_memory(&self, session_id: &str, content: &str) -> Result<()>;

    /// Retrieve the top-`limit` most semantically relevant memories for the query.
    async fn search_memories(
        &self,
        session_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<String>>;
}
