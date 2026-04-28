#![cfg(feature = "llama-cpp")]

use crate::error::AmbiError;
use serde::Deserialize;
use std::path::Path;

/// Configuration settings for the local `llama.cpp` inference engine.
///
/// This struct defines the hardware acceleration, context windows, sampling penalties,
/// and memory allocation parameters required to initialize a local GGUF model via
/// the `llama-cpp-2` bindings.
///
/// # Examples
///
/// ```rust
/// use ambi::llm::providers::llama_cpp::LlamaEngineConfig;
///
/// let config = LlamaEngineConfig {
///     model_path: "./models/llama-3-8b.gguf".to_string(),
///     max_tokens: 4096,
///     buffer_size: 32,
///     use_gpu: true,
///     n_gpu_layers: 100, // Offload all layers to GPU
///     n_ctx: 8192,
///     n_tokens: 512,
///     n_seq_max: 1,
///     penalty_last_n: 64,
///     penalty_repeat: 1.1,
///     penalty_freq: 0.0,
///     penalty_present: 0.0,
///     temp: 0.7,
///     top_p: 0.9,
///     seed: 42,
///     min_keep: 1,
/// };
/// ```
///
#[derive(Debug, Deserialize, Clone)]
pub struct LlamaEngineConfig {
    pub model_path: String,

    /// External vision projector model path (e.g., mmproj-model-f16.gguf).
    /// Used for decoupled multimodal architectures like LLaVA.
    pub mmproj_path: Option<String>,

    /// Indicates whether the main LLM has native, integrated vision capabilities
    /// (e.g., Qwen2-VL, Llama-3.2-Vision), requiring no external mmproj file.
    #[serde(default)]
    pub integrated_vision: bool,

    pub max_tokens: i32,
    pub buffer_size: usize,
    pub use_gpu: bool,
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

impl LlamaEngineConfig {
    pub fn validate(&self) -> crate::error::Result<()> {
        if !Path::new(&self.model_path).exists() {
            return Err(AmbiError::EngineError(format!(
                "Local model file does not exist: {}",
                self.model_path
            )));
        }

        // Validate external vision projector if specified
        if let Some(path) = &self.mmproj_path {
            if !Path::new(path).exists() {
                return Err(AmbiError::EngineError(format!(
                    "Local vision projector (mmproj) file does not exist: {}",
                    path
                )));
            }
        }

        if self.n_ctx == 0 {
            return Err(AmbiError::EngineError(
                "Context n_ctx cannot be 0.".to_string(),
            ));
        }
        if self.temp < 0.0 || self.temp > 2.0 {
            return Err(AmbiError::EngineError(
                "Temperature must be between 0.0 and 2.0".to_string(),
            ));
        }
        Ok(())
    }
}
