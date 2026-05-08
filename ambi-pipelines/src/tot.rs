//! Tree of Thoughts (ToT) pipeline.
//!
//! Provides breadth-first beam search exploration for complex, multi-step
//! reasoning tasks. Leverages lock-free state cloning for strict branch isolation
//! and automatic memory pruning for dead ends.

pub mod bfs;
pub mod traits;

pub use bfs::BfsBeamSearchPipeline;
pub use traits::{Evaluation, TotEvaluator, TotExpander};
