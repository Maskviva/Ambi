//! Self-Consistency pipeline implementation.
//!
//! Runs multiple independent reasoning branches in parallel, extracts answers
//! from each, and selects the most frequent one via majority voting.

use super::traits::{Aggregator, AnswerExtractor, MajorityVoting};
use ambi::agent::core::{Agent, AgentState};
use ambi::agent::pipeline::{ChatRunner, Pipeline};
use ambi::error::{AmbiError, Result};
use ambi::ContentPart;

use futures::future::join_all;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

/// Self-Consistency pipeline: runs multiple branches in parallel and picks the most common answer.
pub struct SelfConsistencyPipeline {
    branches: usize,
    extractor: Arc<dyn AnswerExtractor>,
    aggregator: Arc<dyn Aggregator>,
    inner_runner_concurrency: usize,
}

impl SelfConsistencyPipeline {
    /// Creates a new pipeline with the given answer extractor.
    pub fn create(extractor: impl AnswerExtractor + 'static) -> Self {
        Self {
            branches: 3,
            extractor: Arc::new(extractor),
            aggregator: Arc::new(MajorityVoting),
            inner_runner_concurrency: 5,
        }
    }

    /// Sets the number of parallel reasoning branches (default: 3).
    pub fn branches(mut self, n: usize) -> Self {
        self.branches = n;
        self
    }

    /// Sets a custom aggregator for combining branch answers (default: majority voting).
    pub fn aggregator(mut self, agg: impl Aggregator + 'static) -> Self {
        self.aggregator = Arc::new(agg);
        self
    }

    /// Sets the maximum concurrency for the inner runner (default: 5).
    pub fn concurrency(mut self, c: usize) -> Self {
        self.inner_runner_concurrency = c;
        self
    }
}

impl Pipeline for SelfConsistencyPipeline {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        if self.branches == 0 {
            return Err(AmbiError::PipelineError("Branches must be >= 1".into()));
        }

        let mut futures = Vec::with_capacity(self.branches);

        for i in 0..self.branches {
            let agent_clone = agent.clone();
            let input_clone = input.clone();
            let runner = ChatRunner::new(self.inner_runner_concurrency);

            let cloned_state = {
                let lock = state.read().await;
                let mut s = AgentState::new(&lock.session_id);
                s.dynamic_context = lock.dynamic_context.clone();
                s.chat_history = lock.chat_history.clone();
                Arc::new(RwLock::new(s))
            };

            futures.push(async move {
                let res = runner
                    .execute(&agent_clone, &cloned_state, input_clone)
                    .await;
                (i, res, cloned_state)
            });
        }

        let results = join_all(futures).await;

        let mut raw_outputs = Vec::new();
        let mut extracted_answers = Vec::new();
        let mut valid_states = Vec::new();

        for (idx, res, branch_state) in results {
            if let Ok(raw_text) = res {
                let extracted = self.extractor.extract(&raw_text);
                raw_outputs.push(raw_text);
                extracted_answers.push(extracted);
                valid_states.push(branch_state);
            } else {
                log::warn!("Self-Consistency branch {} failed.", idx);
            }
        }

        if raw_outputs.is_empty() {
            return Err(AmbiError::PipelineError("All branches failed.".into()));
        }

        let winner_idx = self
            .aggregator
            .aggregate(&extracted_answers)
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        let winning_raw_text = raw_outputs[winner_idx].clone();
        let winning_state = &valid_states[winner_idx];

        {
            let mut main_lock = state.write().await;
            let winner_lock = winning_state.read().await;
            main_lock.chat_history = winner_lock.chat_history.clone();
        }

        Ok(winning_raw_text)
    }

    async fn execute_stream(
        &self,
        _agent: &Agent,
        _state: &Arc<RwLock<AgentState>>,
        _input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        Err(AmbiError::PipelineError(
            "Self-Consistency Pipeline does not support streaming.".into(),
        ))
    }
}
