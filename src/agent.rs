// src/agent.rs

//! The core Agent domain, managing configurations, history, tools, and execution pipelines.

/// Core agent entities, builders, and memory states.
pub mod core;
/// Execution pipelines for chat and tool interactions.
pub mod pipeline;
/// Output streaming post-processors and formatters.
pub mod processor;
/// Tool registries, dynamic invocation managers, and parsers.
pub mod tool;

pub use self::core::{Agent, AgentState};
