//! Chain of Thought (CoT) and Self-Consistency pipelines.
//!
//! Provides high-concurrency sampling and majority voting mechanisms to greatly
//! improve the LLM's logical reasoning and mathematical accuracy.

pub mod self_consistency;
pub mod traits;

pub use self_consistency::SelfConsistencyPipeline;
pub use traits::{Aggregator, AnswerExtractor, MajorityVoting, PatternExtractor};
