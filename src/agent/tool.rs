// src/agent/tool.rs

//! Tool registries, dynamic invocation managers, and parsers.

/// Dynamic tool registry and invocation management.
pub mod manager;
/// Parsers for LLM-generated tool call syntax.
pub mod parser;

pub use manager::ToolManager;
pub use parser::{DefaultToolParser, TagToolParser};
