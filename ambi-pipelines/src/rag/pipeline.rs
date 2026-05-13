//! Standard RAG pipeline implementation.
//!
//! Retrieves relevant documents, packs them into the context, and delegates
//! the final chat invocation to an inner `ChatRunner`.

use super::packing::ContextPacker;
use super::retriever::Retriever;
use ambi::agent::core::{Agent, AgentState};
use ambi::agent::pipeline::{ChatRunner, Pipeline};
use ambi::error::{AmbiError, Result};
use ambi::ContentPart;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

pub struct StandardRagPipeline {
    retriever: Arc<dyn Retriever>,
    max_context_tokens: usize,
    inner_runner: ChatRunner,
}

impl StandardRagPipeline {
    pub fn create(retriever: impl Retriever + 'static) -> Self {
        Self {
            retriever: Arc::new(retriever),
            max_context_tokens: 4096,
            inner_runner: ChatRunner::default(),
        }
    }

    pub fn max_context_tokens(mut self, max: usize) -> Self {
        self.max_context_tokens = max;
        self
    }

    pub fn concurrency(mut self, concurrency: usize) -> Self {
        self.inner_runner.maximum_concurrency = concurrency;
        self
    }

    fn extract_query_text(input: &[ContentPart]) -> String {
        input
            .iter()
            .filter_map(|p| {
                if let ContentPart::Text { text } = p {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    async fn prepare_context(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: &[ContentPart],
    ) -> Result<()> {
        let query = Self::extract_query_text(input);
        if query.trim().is_empty() {
            return Ok(());
        }

        let docs = self
            .retriever
            .retrieve(&query)
            .await
            .map_err(|e| AmbiError::PipelineError(format!("RAG Retrieval failed: {}", e)))?;

        let packed_context =
            ContextPacker::pack(&agent.get_llama_engine(), docs, self.max_context_tokens)?;

        let mut lock = state.write().await;
        if !packed_context.is_empty() {
            lock.set_dynamic_context(&packed_context);
        } else {
            lock.clear_dynamic_context();
        }

        Ok(())
    }
}

impl Pipeline for StandardRagPipeline {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        self.prepare_context(agent, state, &input).await?;
        self.inner_runner.execute(agent, state, input).await
    }

    async fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        self.prepare_context(agent, state, &input).await?;
        self.inner_runner.execute_stream(agent, state, input).await
    }
}
