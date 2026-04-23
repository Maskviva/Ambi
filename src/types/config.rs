// src/types/config.rs

mod agent;
mod llama_cpp;
mod open_ai;

pub use agent::AgentConfig;

#[cfg(feature = "llama-cpp")]
pub use llama_cpp::LlamaEngineConfig;

#[cfg(feature = "openai-api")]
pub use open_ai::OpenAIEngineConfig;
