pub mod chat_template;
pub mod handler;

pub mod engine;

pub use engine::llama_cpp_2::llama_cpp_2_config::LlamaEngineConfig;
pub use engine::openai_api::openai_api_config::OpenAIEngineConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum EngineConfig {
    #[cfg(feature = "llama-cpp")]
    Llama(LlamaEngineConfig),

    #[cfg(feature = "openai-api")]
    OpenAI(OpenAIEngineConfig),
}
