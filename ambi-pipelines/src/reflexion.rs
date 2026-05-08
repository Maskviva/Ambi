//! Reflexion and Self-Healing pipeline.
//!
//! Integrates an Actor-Evaluator loop with `ambi-memory`. Allows the agent to
//! execute, evaluate its own output, and recursively correct itself by writing
//! and reading "reflections" (critiques) from a persistent memory store.

pub mod pipeline;
pub mod traits;

pub use pipeline::ReflexionPipeline;
pub use traits::{EvaluationResult, Evaluator};
