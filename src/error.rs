// src/error.rs

//! Error definitions and result types for the Ambi framework.

use std::sync::Arc;
use thiserror::Error;

/// The kind of failure behind an [`AmbiError::EngineError`].
///
/// Added so callers can tell a bad request apart from a transport failure, a model that failed to
/// load, or a tokenization problem — and therefore whether retrying makes sense — without parsing
/// the message text. The public error still renders exactly as before; only the kind is new.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineErrorKind {
    /// The request itself is invalid and will fail the same way if retried unchanged.
    InvalidRequest,
    /// A network or remote-endpoint failure. Retrying may succeed.
    Transport,
    /// The model or its runtime could not be initialized. Retrying will not help.
    ModelLoad,
    /// Tokenization failed.
    Tokenize,
    /// Anything else, including failures from before this classification existed.
    Other,
}

impl EngineErrorKind {
    /// Whether repeating the same call could plausibly succeed.
    ///
    /// Only transport failures are transient. A request the provider rejected, a model that failed
    /// to load, and a tokenization error will all fail again unchanged.
    pub fn is_retryable(self) -> bool {
        matches!(self, EngineErrorKind::Transport)
    }
}

/// Represents all possible errors that can occur within the Ambi framework.
#[derive(Error, Debug, Clone)]
pub enum AmbiError {
    /// Triggered when the underlying LLM engine encounters an initialization or inference failure.
    ///
    /// The message renders exactly as it always has. The second field classifies the failure so
    /// callers can decide whether to retry instead of guessing from the text; it defaults to
    /// [`EngineErrorKind::Other`] wherever an older call site has not been classified yet.
    #[error("LLM Engine Error: {0}")]
    EngineError(String, EngineErrorKind),

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

impl AmbiError {
    /// Builds an [`AmbiError::EngineError`] of the given kind.
    pub fn engine_kind(kind: EngineErrorKind, message: impl Into<String>) -> Self {
        AmbiError::EngineError(message.into(), kind)
    }

    /// Builds an [`AmbiError::EngineError`] classified as [`EngineErrorKind::Other`].
    ///
    /// This is the shape every engine failure had before failures were classified, so existing call
    /// sites keep their meaning. Sites that know better should name the kind explicitly.
    pub fn engine(message: impl Into<String>) -> Self {
        Self::engine_kind(EngineErrorKind::Other, message)
    }
}

/// A specialized `Result` type for Ambi operations.
pub type Result<T> = std::result::Result<T, AmbiError>;
