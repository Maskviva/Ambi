// bindings/node/src/lib.rs
//
//! # Ambi Node.js Bindings
//!
//! A thin, type-complete bridge from the Ambi Rust framework into Node.js.
//! Instead of wrapping the crate behind a handful of convenience methods —
//! which inevitably caps what JS users can express — we expose the full Rust
//! type system and API surface. The result is a binding that feels closer to
//! the native experience than to a typical "lightweight SDK wrapper."
//!
//! ## Modules
//!
//! - **types**: Every core type that matters — `ContentPart`, `Message`,
//!   `ToolDefinition`, `LLMRequest`. JS users construct them the same way
//!   Rust users would.
//! - **config**: All knobs and dials the framework exposes — eviction policies,
//!   chat template formats, OpenAI connection settings.
//! - **engine**: The engine itself — synchronous chat, streaming, context
//!   management, entropy evaluation.
//! - **agent**: The fluent builder blueprint (`Agent`) and per-conversation
//!   memory (`AgentState`).
//! - **pipeline**: The default ReAct loop (`ChatRunner`), including streaming
//!   callbacks for real-time token delivery.

#![deny(clippy::all)]

pub mod agent;
pub mod config;
pub mod engine;
pub mod pipeline;
pub mod types;
