//! # Ambi Pipelines
//!
//! Advanced execution workflows and cognitive architectures for the Ambi AI framework.
//!
//! This crate extends the base `Pipeline` trait to support sophisticated interaction
//! patterns such as Retrieval-Augmented Generation (RAG), Chain of Thought (CoT),
//! Tree of Thoughts (ToT), and Reflexion.

/// Retrieval-Augmented Generation (RAG) implementation.
pub mod rag;

/// Chain of Thought (CoT) prompting pipeline.
pub mod cot;

/// Reflexion and Self-Healing pipeline.
pub mod reflexion;

/// Tree of Thoughts (ToT) pipeline.
pub mod tot;

// Export the default official runner as React (Reason + Act) alias
pub use ambi::ChatRunner as ReactPipeline;
