pub mod engine;
pub mod providers;
pub mod template;
mod tokenizer;

use crate::types::config;

pub use engine::{LLMEngine, LLMEngineTrait};
pub use template::{ChatTemplate, ChatTemplateType};

pub enum LLMEngineConfig {
    #[cfg(feature = "openai-api")]
    OpenAI(config::OpenAIEngineConfig),
    #[cfg(feature = "llama-cpp")]
    Llama(config::LlamaEngineConfig),
}
