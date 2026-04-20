use serde::Deserialize;

#[cfg(feature = "llama-cpp")]
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

#[cfg(feature = "llama-cpp")]
impl LlamaEngineConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if !std::path::Path::new(&self.model_path).exists() {
            return Err(anyhow::anyhow!(
                "Local model file does not exist: {}",
                self.model_path
            ));
        }
        if self.n_ctx == 0 {
            return Err(anyhow::anyhow!("Context n_ctx cannot be 0."));
        }
        if self.temp < 0.0 || self.temp > 2.0 {
            return Err(anyhow::anyhow!("Temperature must be between 0.0 and 2.0"));
        }
        Ok(())
    }
}
