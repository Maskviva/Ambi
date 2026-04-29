// src/lib.rs

//! # Ambi: A Flexible, Multi-Backend AI Agent Framework
//!
//! `ambi` is an incredibly fast, highly modular, and fully customizable AI agent framework
//! written entirely in Rust. It acts as the bridge between Large Language Models (LLMs) and
//! your Rust application, providing a robust execution loop, tool-calling capabilities, and
//! deterministic context management.
//!
//! ## Core Features
//!
//! - **Multi-Backend Support**: Seamlessly switch between cloud APIs (OpenAI format, DeepSeek, etc.)
//!   and hyper-optimized local inference via `llama.cpp` using static Cargo features.
//! - **Deterministic Tool Calling**: Expose your Rust functions to the LLM. Features strict
//!   timeout controls, distinction between idempotent and non-idempotent operations, and
//!   graceful JSON truncation recovery.
//! - **Robust Context Eviction**: Never worry about `max_tokens` overflow again. Ambi uses a
//!   deterministic FIFO algorithm to prune conversation history while preserving critical context.
//! - **Multimodal (Vision) Ready**: Built-in support for processing images, whether through
//!   native integrated models (e.g., Qwen2-VL) or external vision projectors (e.g., LLaVA).
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ambi::{Agent, AgentState, ChatRunner};
//! use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! #[tokio::main]
//! async fn main() -> ambi::error::Result<()> {
//!     // 1. Initialize the configuration
//!     let config = OpenAIEngineConfig {
//!         api_key: "your-api-key".to_string(),
//!         base_url: "https://api.openai.com/v1".to_string(),
//!         model_name: "gpt-4o".to_string(),
//!         temp: 0.7,
//!         top_p: 0.95,
//!     };
//!
//!     // 2. Build the Agent
//!     let agent = Agent::make(ambi::LLMEngineConfig::OpenAI(config))
//!         .await?
//!         .preamble("You are a helpful and concise assistant.")
//!         .with_standard_formatting();
//!
//!     // 3. Initialize Conversation State
//!     let state = Arc::new(RwLock::new(AgentState::new()));
//!     let runner = ChatRunner;
//!
//!     // 4. Execute the pipeline
//!     let response = runner.chat(&agent, &state, "Hello, world!").await?;
//!     println!("Assistant: {}", response);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Runtime Requirements
//!
//! Ambi requires the **Tokio** async runtime with the `rt-multi-thread` feature.
//! The following is the minimum setup in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
//! ```

#![warn(missing_docs)] // Enforce documentation across the crate

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

/// Agent Framework Core
pub mod agent;

/// Configuration
pub mod config;

/// Error Handling
pub mod error;

/// LLM Engine
pub mod llm;

/// Types
pub mod types;

pub use agent::core::{Agent, AgentState};
pub use agent::pipeline::chat_runner::ChatRunner;
pub use llm::{LLMEngine, LLMEngineConfig};
pub use types::{ContentPart, Message};
