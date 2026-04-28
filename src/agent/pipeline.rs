// src/agent/pipeline.rs
pub mod chat_runner;

use crate::agent::core::{Agent, AgentState};
use crate::error::Result;
use crate::ContentPart;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

pub trait Pipeline: Send + Sync {
    fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> impl std::future::Future<Output = Result<String>> + Send;

    fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> impl std::future::Future<Output = Result<Pin<Box<ReceiverStream<Result<String>>>>>> + Send;
}

pub use chat_runner::ChatRunner;
