//! Error and result types for the ambi-memory subsystem.
//!
//! Defines specialised error variants for KV storage, semantic search,
//! serialisation, and summary generation backends.

use thiserror::Error;

/// All possible errors that can occur during memory operations.
#[derive(Error, Debug)]
pub enum MemoryError {
    /// An unrecoverable failure in the underlying storage backend.
    #[error("Memory Backend Error: {0}")]
    BackendError(String),

    /// A serialisation or deserialisation failure.
    #[error("Memory Serialization Error: {0}")]
    SerializationError(String),

    /// The requested key does not exist in the store.
    #[error("Memory Key Not Found: {0}")]
    KeyNotFound(String),

    /// An embedding or vector-search operation failed.
    #[error("Embedding or Semantic Search Error: {0}")]
    SemanticError(String),

    /// A summarisation LLM call or compression step failed.
    #[error("Summary Generation Error: {0}")]
    SummaryError(String),
}

/// A specialised `Result` type for memory operations.
pub type Result<T> = std::result::Result<T, MemoryError>;
