// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AmbiError {
    #[error("LLM Engine Error: {0}")]
    EngineError(String),

    #[error("Agent Error: {0}")]
    AgentError(String),

    #[error("Tool Execution Error: {0}")]
    ToolError(String),

    #[error("Context or Prompt Formatting Error: {0}")]
    ContextError(String),

    #[error("Pipeline Execution Error: {0}")]
    PipelineError(String),

    #[error("Maximum iterations ({0}) reached without a final answer")]
    MaxIterationsReached(usize),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AmbiError>;
