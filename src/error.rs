// src/error.rs

//! Error definitions and result types for the Ambi framework.

use std::sync::Arc;
use thiserror::Error;

/// Represents all possible errors that can occur within the Ambi framework.
#[derive(Error, Debug, Clone)]
pub enum AmbiError {
    /// Triggered when the underlying LLM engine encounters an initialization or inference failure.
    #[error("LLM Engine Error: {0}")]
    EngineError(String),

    /// Triggered when the Agent encounters a logical or state management error.
    #[error("Agent Error: {0}")]
    AgentError(String),

    /// Triggered when a tool execution fails or times out.
    #[error("Tool Execution Error: {0}")]
    ToolError(String),

    /// Triggered when the context is too large or prompt formatting fails.
    #[error("Context or Prompt Formatting Error: {0}")]
    ContextError(String),

    /// Triggered when the pipeline stream is unexpectedly interrupted or disconnected.
    #[error("Pipeline Execution Error: {0}")]
    PipelineError(String),

    /// Triggered when the Agent reaches the maximum allowed iterations without providing a final answer.
    #[error("Maximum iterations ({0}) reached without a final answer")]
    MaxIterationsReached(usize),

    /// A generic wrapper for any other unclassified `anyhow::Error`.
    #[error(transparent)]
    Other(#[from] Arc<anyhow::Error>),
}

/// A specialized `Result` type for Ambi operations.
pub type Result<T> = std::result::Result<T, AmbiError>;
