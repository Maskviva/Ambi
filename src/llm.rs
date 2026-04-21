pub mod engine;
pub mod providers;
pub mod template;

use crate::types::message::Message;

pub use engine::{LLMEngine, LLMEngineTrait};
pub use template::{ChatTemplate, ChatTemplateType};

#[derive(Clone, Debug)]
pub struct LLMRequest {
    pub system_prompt: String,
    pub history: Vec<Message>,
    pub tool_prompt: String,
    pub formatted_prompt: String,
}

pub enum LLMEngineConfig {
    #[cfg(feature = "openai-api")]
    OpenAI(providers::openai_api::OpenAIEngineConfig),
    #[cfg(feature = "llama-cpp")]
    Llama(providers::llama_cpp::LlamaEngineConfig),
}
