//! Breadth-First Search (BFS) beam search pipeline for Tree of Thoughts.

use super::traits::{Evaluation, TotEvaluator, TotExpander};
use ambi::ContentPart;
use ambi::agent::core::{Agent, AgentState};
use ambi::agent::pipeline::{ChatRunner, Pipeline};
use ambi::error::{AmbiError, Result};

use futures::future::join_all;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

/// Breadth-First Beam Search pipeline for Tree of Thoughts reasoning.
///
/// Expands the most promising thoughts at each depth, evaluates them, and
/// prunes low-scoring branches to maintain a fixed beam width.
pub struct BfsBeamSearchPipeline {
    expander: Arc<dyn TotExpander>,
    evaluator: Arc<dyn TotEvaluator>,
    branching_factor_k: usize,
    beam_width_b: usize,
    max_steps: usize,
    inner_runner: ChatRunner,
}

impl BfsBeamSearchPipeline {
    /// Creates a new ToT pipeline with the given expander and evaluator.
    pub fn create(
        expander: impl TotExpander + 'static,
        evaluator: impl TotEvaluator + 'static,
    ) -> Self {
        Self {
            expander: Arc::new(expander),
            evaluator: Arc::new(evaluator),
            branching_factor_k: 3,
            beam_width_b: 2,
            max_steps: 5,
            inner_runner: ChatRunner::new(10),
        }
    }

    /// Sets the branching factor (number of candidate thoughts per node, default: 3).
    pub fn branching_factor(mut self, k: usize) -> Self {
        self.branching_factor_k = k;
        self
    }

    /// Sets the beam width (number of top-scoring branches to keep, default: 2).
    pub fn beam_width(mut self, b: usize) -> Self {
        self.beam_width_b = b;
        self
    }

    /// Sets the maximum exploration depth (default: 5).
    pub fn max_steps(mut self, steps: usize) -> Self {
        self.max_steps = steps;
        self
    }

    /// Sets the inner runner's maximum concurrency (default: 10).
    pub fn concurrency(mut self, c: usize) -> Self {
        self.inner_runner.maximum_concurrency = c;
        self
    }

    async fn clone_state(state: &Arc<RwLock<AgentState>>) -> Arc<RwLock<AgentState>> {
        let lock = state.read().await;
        let mut new_state = AgentState::new(&lock.session_id);
        new_state.dynamic_context = lock.dynamic_context.clone();
        new_state.chat_history = lock.chat_history.clone();
        Arc::new(RwLock::new(new_state))
    }
}

impl Pipeline for BfsBeamSearchPipeline {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        self.inner_runner.execute(agent, state, input).await?;

        let mut frontier: Vec<(Arc<RwLock<AgentState>>, f32)> = vec![(Arc::clone(state), 1.0)];

        for step in 0..self.max_steps {
            log::info!(
                "ToT Pipeline: Exploring Depth {}/{}",
                step + 1,
                self.max_steps
            );

            let mut expansion_futures = Vec::new();

            for (node_state, _parent_score) in frontier.drain(..) {
                let expander = Arc::clone(&self.expander);
                let agent_clone = agent.clone();
                let runner_clone = self.inner_runner.clone();
                let k = self.branching_factor_k;

                expansion_futures.push(async move {
                    let thoughts = expander
                        .expand(&agent_clone, &node_state, &runner_clone, k)
                        .await
                        .unwrap_or_default();
                    (node_state, thoughts)
                });
            }

            let expanded_nodes = join_all(expansion_futures).await;
            let mut evaluation_futures = Vec::new();

            for (parent_state, thoughts) in expanded_nodes {
                for thought in thoughts {
                    let evaluator = Arc::clone(&self.evaluator);
                    let agent_clone = agent.clone();
                    let runner_clone = self.inner_runner.clone();

                    let child_state = Self::clone_state(&parent_state).await;
                    let thought_text = format!("Step {}: {}", step + 1, thought);

                    evaluation_futures.push(async move {
                        let eval_result = evaluator
                            .evaluate(&agent_clone, &child_state, &runner_clone, &thought_text)
                            .await
                            .unwrap_or(Evaluation::Invalid);

                        (child_state, thought_text, eval_result)
                    });
                }
            }

            let evaluated_children = join_all(evaluation_futures).await;
            let mut next_candidates = Vec::new();

            for (child_state, thought_text, eval_result) in evaluated_children {
                match eval_result {
                    Evaluation::Terminal(final_answer) => {
                        let mut main_lock = state.write().await;
                        let child_lock = child_state.read().await;
                        main_lock.chat_history = child_lock.chat_history.clone();

                        return Ok(final_answer);
                    }
                    Evaluation::Intermediate(score) => {
                        child_state.write().await.append_dynamic_context(&format!(
                            "\n[Thought Adopted]: {}",
                            thought_text
                        ));
                        next_candidates.push((child_state, score));
                    }
                    Evaluation::Invalid => {}
                }
            }

            if next_candidates.is_empty() {
                return Err(AmbiError::PipelineError(
                    "ToT Exploration failed: All branches evaluated as Invalid.".into(),
                ));
            }

            next_candidates
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            frontier = next_candidates
                .into_iter()
                .take(self.beam_width_b)
                .collect();
        }

        Err(AmbiError::PipelineError(format!(
            "ToT Exploration exhausted max steps ({}) without reaching a Terminal state.",
            self.max_steps
        )))
    }

    async fn execute_stream(
        &self,
        _agent: &Agent,
        _state: &Arc<RwLock<AgentState>>,
        _input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        Err(AmbiError::PipelineError(
            "ToT Pipeline does not support streaming.".into(),
        ))
    }
}
