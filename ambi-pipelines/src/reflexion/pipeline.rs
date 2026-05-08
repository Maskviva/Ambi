//! Reflexion pipeline implementation — Actor-Evaluator loop with persistent critique memory.

use super::traits::Evaluator;
use ambi::ContentPart;
use ambi::agent::core::{Agent, AgentState};
use ambi::agent::pipeline::{ChatRunner, Pipeline};
use ambi::error::{AmbiError, Result};

use ambi_memory::{AgentStateMemoryExt, KvMemoryProvider};

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

/// Reflexion pipeline: generates a response, evaluates it, and recursively corrects via stored critiques.
pub struct ReflexionPipeline {
    evaluator: Arc<dyn Evaluator>,
    memory_provider: Arc<dyn KvMemoryProvider>,
    max_retries: usize,
    inner_runner: ChatRunner,
}

impl ReflexionPipeline {
    /// Creates a new Reflexion pipeline with the given evaluator and memory provider.
    pub fn create(
        evaluator: impl Evaluator + 'static,
        memory_provider: impl KvMemoryProvider + 'static,
    ) -> Self {
        Self {
            evaluator: Arc::new(evaluator),
            memory_provider: Arc::new(memory_provider),
            max_retries: 3,
            inner_runner: ChatRunner::default(),
        }
    }

    /// Sets the maximum number of retry attempts (default: 3).
    pub fn max_retries(mut self, max: usize) -> Self {
        self.max_retries = max;
        self
    }

    /// Sets the inner runner's maximum concurrency.
    pub fn concurrency(mut self, c: usize) -> Self {
        self.inner_runner.maximum_concurrency = c;
        self
    }
}

impl Pipeline for ReflexionPipeline {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        {
            let mut lock = state.write().await;
            let _ = lock.recall_kv_into_context(&*self.memory_provider).await;
        }

        let mut current_attempt = 0;

        loop {
            current_attempt += 1;
            log::info!(
                "Reflexion: Attempt {}/{}",
                current_attempt,
                self.max_retries
            );

            let response = self
                .inner_runner
                .execute(agent, state, input.clone())
                .await?;

            let evaluation = self
                .evaluator
                .evaluate(&response)
                .await
                .map_err(|e| AmbiError::PipelineError(format!("Evaluator error: {}", e)))?;

            if evaluation.is_pass {
                return Ok(response);
            }

            let critique = evaluation
                .critique
                .unwrap_or_else(|| "Failed criteria.".to_string());
            if current_attempt >= self.max_retries {
                return Ok(response);
            }

            let mut lock = state.write().await;
            let memory_key = format!("critique_{}", chrono::Utc::now().timestamp_millis());
            let _ = lock
                .remember_kv(&*self.memory_provider, &memory_key, &critique)
                .await;

            let formatted_critique =
                format!("\n[EVALUATOR FEEDBACK]: {}\nPlease correct this.", critique);
            lock.append_dynamic_context(&formatted_critique);
        }
    }

    async fn execute_stream(
        &self,
        _agent: &Agent,
        _state: &Arc<RwLock<AgentState>>,
        _input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        Err(AmbiError::PipelineError(
            "Reflexion does not support streaming.".into(),
        ))
    }
}
