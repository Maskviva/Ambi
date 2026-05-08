//! Trait and type definitions for the Reflexion evaluation loop.

use async_trait::async_trait;

/// The result of evaluating an agent's response.
pub struct EvaluationResult {
    /// Whether the response passed the quality bar.
    pub is_pass: bool,
    /// An optional textual critique explaining the failure (used for self-correction).
    pub critique: Option<String>,
}

/// Evaluates an agent's output and returns a pass/fail verdict with optional feedback.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Evaluator: Send + Sync {
    async fn evaluate(
        &self,
        response: &str,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send + Sync>>;
}
