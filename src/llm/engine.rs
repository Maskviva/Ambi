#[cfg(feature = "llama-cpp")]
pub mod llama_cpp_2;

#[cfg(feature = "openai-api")]
pub mod openai_api;

#[cfg(feature = "llama-cpp")]
pub use llama_cpp_2::llama_cpp_2_bridging::LlamaEngine;
#[cfg(feature = "llama-cpp")]
pub use llama_cpp_2::llama_cpp_2_config::LlamaEngineConfig;
use serde::Deserialize;

use crate::OpenAIEngineConfig;
#[cfg(feature = "openai-api")]
pub use openai_api::openai_api_bridging::OpenAIEngine;

#[derive(Debug, Deserialize, Clone)]
pub enum LLMEngineConfig {
    #[cfg(feature = "llama-cpp")]
    Llama(LlamaEngineConfig),

    #[cfg(feature = "openai-api")]
    OpenAI(OpenAIEngineConfig),
}
