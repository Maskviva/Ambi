//! Abstract retriever trait for pluggable RAG backends.

use super::document::Document;
use async_trait::async_trait;

/// Generic document retriever interface.
///
/// Implementations can wrap vector databases, search engines, or any other
/// information retrieval system.
#[async_trait]
pub trait Retriever: Send + Sync {
    /// Retrieve documents relevant to the given query.
    async fn retrieve(
        &self,
        query: &str,
    ) -> Result<Vec<Document>, Box<dyn std::error::Error + Send + Sync>>;
}
