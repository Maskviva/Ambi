// src/llm/providers/llama_cpp/config.rs

//! Configuration properties for local Llama.cpp inference.

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
    /// The file path to the local `.gguf` model weight.
    pub model_path: String,

    /// External vision projector model path (e.g., mmproj-model-f16.gguf).
    /// Used for decoupled multimodal architectures like LLaVA.
    pub mmproj_path: Option<String>,

    /// Indicates whether the main LLM has native, integrated vision capabilities.
    #[serde(default)]
    pub integrated_vision: bool,

    /// The maximum number of tokens to predict.
    pub max_tokens: i32,
    /// Batch buffer size for piece decoding.
    pub buffer_size: usize,
    /// Whether to offload layers to the GPU.
    pub use_gpu: bool,
    /// Number of layers to offload to the GPU.
    pub n_gpu_layers: u32,
    /// The length of the context window.
    pub n_ctx: u32,
    /// Batch size for prompt processing.
    pub n_tokens: usize,
    /// Maximum sequences allowed in a batch.
    pub n_seq_max: i32,
    /// Number of past tokens to consider for penalties.
    pub penalty_last_n: i32,
    /// Repetition penalty factor.
    pub penalty_repeat: f32,
    /// Frequency penalty factor.
    pub penalty_freq: f32,
    /// Presence penalty factor.
    pub penalty_present: f32,
    /// The sampling temperature (0.0 to 2.0).
    pub temp: f32,
    /// The top-p (nucleus) sampling threshold.
    pub top_p: f32,
    /// The RNG seed for deterministic generation.
    pub seed: u32,
    /// Min-keep sampling boundary.
    pub min_keep: usize,
}

impl LlamaEngineConfig {
    /// Validates the file paths and parameter bounds before initialization.
    pub fn validate(&self) -> crate::error::Result<()> {
        if !Path::new(&self.model_path).exists() {
            return Err(AmbiError::engine_kind(
                crate::error::EngineErrorKind::ModelLoad,
                format!("Local model file does not exist: {}", self.model_path),
            ));
        }

        // Validate external vision projector if specified
        if let Some(path) = &self.mmproj_path {
            if !Path::new(path).exists() {
                return Err(AmbiError::engine_kind(
                    crate::error::EngineErrorKind::ModelLoad,
                    format!(
                        "Local vision projector (mmproj) file does not exist: {}",
                        path
                    ),
                ));
            }
        }

        if self.n_ctx == 0 {
            return Err(AmbiError::engine_kind(
                crate::error::EngineErrorKind::InvalidRequest,
                "Context n_ctx cannot be 0.".to_string(),
            ));
        }
        // A prompt batch larger than the context can never be evaluated, and a non-positive
        // generation budget would make every request fail after the model has already loaded.
        if self.n_tokens == 0 || self.n_tokens > self.n_ctx as usize {
            return Err(AmbiError::engine_kind(
                crate::error::EngineErrorKind::InvalidRequest,
                format!(
                    "Prompt batch n_tokens ({}) must be between 1 and n_ctx ({}).",
                    self.n_tokens, self.n_ctx
                ),
            ));
        }
        if self.max_tokens <= 0 {
            return Err(AmbiError::engine_kind(
                crate::error::EngineErrorKind::InvalidRequest,
                format!("max_tokens ({}) must be greater than 0.", self.max_tokens),
            ));
        }
        if self.temp < 0.0 || self.temp > 2.0 {
            return Err(AmbiError::engine_kind(
                crate::error::EngineErrorKind::InvalidRequest,
                "Temperature must be between 0.0 and 2.0".to_string(),
            ));
        }
        Ok(())
    }

    /// Starts a configuration for the model at `model_path`, filling in conservative defaults.
    ///
    /// The defaults target a CPU build with a modest context, so a caller only has to name the
    /// model file and then override whatever their hardware actually supports.
    pub fn builder(model_path: impl Into<String>) -> LlamaEngineConfigBuilder {
        LlamaEngineConfigBuilder {
            config: Self {
                model_path: model_path.into(),
                mmproj_path: None,
                integrated_vision: false,
                max_tokens: 512,
                buffer_size: 32,
                use_gpu: false,
                n_gpu_layers: 0,
                n_ctx: 2048,
                n_tokens: 512,
                n_seq_max: 1,
                penalty_last_n: 64,
                penalty_repeat: 1.1,
                penalty_freq: 0.0,
                penalty_present: 0.0,
                temp: 0.7,
                top_p: 0.9,
                seed: 42,
                min_keep: 1,
            },
        }
    }
}

/// Builds a [`LlamaEngineConfig`] without requiring all of its fields up front.
pub struct LlamaEngineConfigBuilder {
    config: LlamaEngineConfig,
}

impl LlamaEngineConfigBuilder {
    /// Sets the external vision projector model path.
    pub fn mmproj_path(mut self, path: impl Into<String>) -> Self {
        self.config.mmproj_path = Some(path.into());
        self
    }

    /// States that the model has native, integrated vision.
    pub fn integrated_vision(mut self, enabled: bool) -> Self {
        self.config.integrated_vision = enabled;
        self
    }

    /// Sets the maximum number of tokens to predict.
    pub fn max_tokens(mut self, max_tokens: i32) -> Self {
        self.config.max_tokens = max_tokens;
        self
    }

    /// Sets the output buffer size for token decoding.
    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.config.buffer_size = buffer_size;
        self
    }

    /// Enables GPU offload and sets how many layers to move.
    pub fn gpu(mut self, n_gpu_layers: u32) -> Self {
        self.config.use_gpu = true;
        self.config.n_gpu_layers = n_gpu_layers;
        self
    }

    /// Sets the context window length.
    pub fn n_ctx(mut self, n_ctx: u32) -> Self {
        self.config.n_ctx = n_ctx;
        self
    }

    /// Sets the batch size for prompt processing.
    pub fn n_tokens(mut self, n_tokens: usize) -> Self {
        self.config.n_tokens = n_tokens;
        self
    }

    /// Sets the sampling temperature.
    pub fn temp(mut self, temp: f32) -> Self {
        self.config.temp = temp;
        self
    }

    /// Sets the nucleus sampling threshold.
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.config.top_p = top_p;
        self
    }

    /// Sets the RNG seed.
    pub fn seed(mut self, seed: u32) -> Self {
        self.config.seed = seed;
        self
    }

    /// Returns the configuration. Bounds are checked later by [`LlamaEngineConfig::validate`].
    pub fn build(self) -> LlamaEngineConfig {
        self.config
    }
}
