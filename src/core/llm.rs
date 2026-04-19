pub mod chat_template;
pub mod formatter;
pub mod handler;

#[cfg(feature = "local")]
pub(crate) mod engine;
#[cfg(feature = "cloud")]
pub mod openai;

#[cfg(feature = "local")]
use crate::core::llm::engine::LlamaEngine;
#[cfg(feature = "cloud")]
use crate::core::llm::openai::OpenAIEngine;

use serde::Deserialize;

pub enum EngineBackend {
    #[cfg(feature = "local")]
    Llama(LlamaEngine),

    #[cfg(feature = "cloud")]
    OpenAI(OpenAIEngine),
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct EngineConfig {
    #[cfg(feature = "local")]
    pub llama: Option<LlamaEngineConfig>,

    #[cfg(feature = "cloud")]
    pub open_ai: Option<OpenAIEngineConfig>,
}

#[cfg(feature = "local")]
#[derive(Debug, Deserialize, Clone)]
pub struct LlamaEngineConfig {
    pub model_path: String,
    pub max_tokens: i32,
    pub buffer_size: usize,
    pub use_gpu: i32,
    pub n_gpu_layers: u32,
    pub n_ctx: u32,
    pub n_tokens: usize,
    pub n_seq_max: i32,
    pub penalty_last_n: i32,
    pub penalty_repeat: f32,
    pub penalty_freq: f32,
    pub penalty_present: f32,
    pub temp: f32,
    pub top_p: f32,
    pub seed: u32,
    pub min_keep: usize,
}

#[cfg(feature = "cloud")]
#[derive(Debug, Deserialize, Clone)]
pub struct OpenAIEngineConfig {
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
    pub temp: f32,
    pub top_p: f32,
}
