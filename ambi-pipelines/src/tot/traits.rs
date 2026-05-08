//! Trait and type definitions for the Tree of Thoughts (ToT) pipeline.

use ambi::agent::core::{Agent, AgentState};
use ambi::agent::pipeline::ChatRunner;
use ambi::error::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The outcome of evaluating a single thought node.
#[derive(Debug, Clone, PartialEq)]
pub enum Evaluation {
    /// The thought is invalid — prune this branch.
    Invalid,
    /// The thought is plausible — keep it with the given score.
    Intermediate(f32),
    /// The thought represents a final answer — terminate search.
    Terminal(String),
}

/// Generates candidate next-step thoughts for a given state.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait TotExpander: Send + Sync {
    /// Generate up to `k` candidate thoughts branching from the current state.
    async fn expand(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        runner: &ChatRunner,
        k: usize,
    ) -> Result<Vec<String>>;
}

/// Evaluates a candidate thought and assigns a score or terminal answer.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait TotEvaluator: Send + Sync {
    /// Evaluate the given `thought` in the context of the current state.
    async fn evaluate(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        runner: &ChatRunner,
        thought: &str,
    ) -> Result<Evaluation>;
}
