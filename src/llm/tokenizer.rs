// src/llm/tokenizer.rs

use crate::error::{AmbiError, Result};

/// Abstract interface for tokenizers.
///
/// The framework requires this interface to be extremely fast and purely synchronous,
/// as it will be called frequently in stream processors and context eviction algorithms.
pub trait TokenizerTrait: Send + Sync {
    /// Synchronously returns the estimated number of tokens consumed by the given text.
    fn count_tokens(&self, text: &str) -> Result<usize>;
}

/// Default tokenizer (based on OpenAI's `cl100k_base` encoding specification).
///
/// **Design Philosophy**:
/// This is an "approximate but extremely fast" solution. Most modern large language
/// models (including Llama-3, Qwen, DeepSeek) use similar vocabularies and BPE algorithms.
/// Using `cl100k_base` as a universal estimator for managing context eviction yields
/// vastly superior performance compared to frequently calling each model's native C++
/// tokenizer at runtime, achieving an excellent time/accuracy tradeoff in engineering.
pub struct DefaultTokenizer {
    bpe: tiktoken_rs::CoreBPE,
}

impl DefaultTokenizer {
    /// Initializes the `cl100k_base` dictionary.
    /// (This operation consumes a few megabytes of resident memory and takes a few
    /// milliseconds to load on the first initialization).
    pub fn make() -> Result<Self> {
        let bpe = tiktoken_rs::cl100k_base().map_err(|e| {
            AmbiError::EngineError(format!("Failed to init default tokenizer: {}", e))
        })?;

        Ok(Self { bpe })
    }
}

impl TokenizerTrait for DefaultTokenizer {
    fn count_tokens(&self, text: &str) -> Result<usize> {
        // Use ordinary text encoding for maximum speed
        Ok(self.bpe.encode_ordinary(text).len())
    }
}
