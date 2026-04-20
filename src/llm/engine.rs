#[cfg(feature = "llama-cpp")]
pub mod llama_cpp_2;

#[cfg(feature = "openai-api")]
pub mod openai_api;

#[cfg(feature = "llama-cpp")]
pub use llama_cpp_2::llama_cpp_2_bridging::LlamaEngine;
#[cfg(feature = "llama-cpp")]
pub use llama_cpp_2::llama_cpp_2_config::LlamaEngineConfig;

#[cfg(feature = "openai-api")]
pub use openai_api::openai_api_bridging::OpenAIEngine;
