//! # Runtime Requirements
//!
//! Ambi requires the **Tokio** async runtime with the `rt-multi-thread` feature.
//! The following is the minimum setup in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
//! ```
#[cfg(any(
    all(feature = "cuda", feature = "vulkan"),
    all(feature = "cuda", feature = "metal"),
    all(feature = "cuda", feature = "rocm"),
    all(feature = "vulkan", feature = "metal"),
    all(feature = "vulkan", feature = "rocm"),
    all(feature = "metal", feature = "rocm")
))]
compile_error!(
    "Cannot enable multiple LLM hardware acceleration backends. Please select only one of: cuda, vulkan, metal, rocm."
);

pub mod agent;
pub mod error;
pub mod llm;
pub mod types;

pub use agent::core::{Agent, AgentState};
pub use agent::pipeline::chat_runner::ChatRunner;
pub use llm::{LLMEngine, LLMEngineConfig};
pub use types::message::{ContentPart, Message};
