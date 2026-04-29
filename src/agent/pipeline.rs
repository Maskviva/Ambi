// src/agent/pipeline.rs

//! Pipeline traits and runner implementations defining the Agent interaction loop.
pub mod chat_runner;

use crate::agent::core::{Agent, AgentState};
use crate::error::Result;
use crate::ContentPart;

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

/// The execution pipeline for managing the Agent's event loop.
///
/// Any struct implementing `Pipeline` can orchestrate the interaction between the LLM,
/// tool resolution, context handling, and user output.
#[cfg(not(target_arch = "wasm32"))]
pub trait Pipeline: Send + Sync {
    /// Synchronous execution mode.
    /// Waits for the entire ReAct-style loop to complete and returns the final synthesized output.
    fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> impl std::future::Future<Output = Result<String>> + Send;

    /// Streaming execution mode.
    /// Yields an asynchronous stream delivering real-time LLM text generation and tool executions.
    fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> impl std::future::Future<Output = Result<Pin<Box<ReceiverStream<Result<String>>>>>> + Send;
}

/// WASM-compatible execution pipeline (removes thread-safety bounds).
#[cfg(target_arch = "wasm32")]
pub trait Pipeline {
    /// Synchronous execution mode for WASM.
    fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> impl std::future::Future<Output = Result<String>>;

    /// Streaming execution mode for WASM.
    fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> impl std::future::Future<Output = Result<Pin<Box<ReceiverStream<Result<String>>>>>>;
}

pub use chat_runner::ChatRunner;
