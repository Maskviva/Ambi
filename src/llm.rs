// src/llm.rs

//! Large Language Model abstractions and provider implementations.

/// Core engine abstractions and wrapper implementations.
pub mod engine;
/// Concrete backend implementations (OpenAI API, Llama.cpp, etc.).
pub mod providers;
/// Tokenization algorithms for context management.
pub mod tokenizer;

pub use engine::{LLMEngine, LLMEngineConfig, LLMEngineTrait};
