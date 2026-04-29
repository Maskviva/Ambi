// src/config/mod.rs

//! The root configuration parameters for defining framework behaviors.

/// Agent-specific configuration structures.
pub mod agent;

pub use agent::{AgentConfig, EvictionStrategy};
