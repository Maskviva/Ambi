// src/llm/tokenizer.rs

use crate::error::Result;

pub trait TokenizerTrait: Send + Sync {
    /// Pure synchronous method, ultra-fast calculation of Token consumption
    fn count_tokens(&self, text: &str) -> Result<usize>;
}

/// Default tokenizer (based on cl100k_base), a general fast approximation scheme for the vast majority of modern large models (GPT-4, Llama-3)
pub struct DefaultTokenizer {
    bpe: tiktoken_rs::CoreBPE,
}

impl DefaultTokenizer {
    pub fn make() -> Self {
        Self {
            bpe: tiktoken_rs::cl100k_base().expect("Failed to initialize default tokenizer"),
        }
    }
}

impl TokenizerTrait for DefaultTokenizer {
    fn count_tokens(&self, text: &str) -> Result<usize> {
        Ok(self.bpe.encode_ordinary(text).len())
    }
}
