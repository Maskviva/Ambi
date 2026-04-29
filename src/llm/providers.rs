// src/llm/providers.rs

//! Concrete implementations of network APIs and local inference engines.

/// Local Llama.cpp inference engine implementation.
#[cfg(feature = "llama-cpp")]
pub mod llama_cpp;

/// OpenAI API and compatible network service implementations.
#[cfg(feature = "openai-api")]
pub mod openai_api;
