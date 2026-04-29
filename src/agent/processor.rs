// src/agent/processor.rs

//! Output streaming post-processors and formatters.

/// Formatters for real-time manipulation of LLM output streams.
pub mod formatter;

pub use self::formatter::{PassThroughFormatter, StandardStreamFormatter};
