// src/types.rs

//! Core data structures, contracts, and type definitions.

mod message;
mod request;
mod template;
mod tool_def;

pub use message::{ContentPart, Message};
pub use request::LLMRequest;
pub use template::{ChatTemplate, ChatTemplateType};
pub use tool_def::{DynTool, StreamFormatter, Tool, ToolCallParser, ToolDefinition, ToolErr};
