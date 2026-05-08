//! Production-ready RAG (Retrieval-Augmented Generation) pipeline.
//!
//! Provides token-aware context packing and a strict Document entity definition
//! to prevent hallucination and token overflow in enterprise applications.

pub mod document;
pub mod packing;
pub mod pipeline;
pub mod retriever;
pub mod semantic_retriever;

pub use document::Document;
pub use pipeline::StandardRagPipeline;
pub use retriever::Retriever;
pub use semantic_retriever::SemanticMemoryRetriever;
