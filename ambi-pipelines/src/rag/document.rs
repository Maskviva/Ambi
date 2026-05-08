//! Document entity used throughout the RAG pipeline.
//!
//! Each `Document` carries an identifier, textual content, a relevance score,
//! and optional metadata for citation tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A retrieved document chunk with score and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier for citation and deduplication.
    pub id: String,
    /// The textual content of the document chunk.
    pub content: String,
    /// Relevance score returned by the retriever (higher = more relevant).
    pub score: f32,
    /// Optional key-value metadata (e.g. source URL, page number).
    pub metadata: HashMap<String, String>,
}

impl Document {
    /// Creates a new document with the given id, content, and score.
    pub fn new(id: impl Into<String>, content: impl Into<String>, score: f32) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            score,
            metadata: HashMap::new(),
        }
    }

    /// Attaches a key-value metadata pair to the document (builder pattern).
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}
