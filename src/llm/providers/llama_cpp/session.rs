// src/llm/providers/llama_cpp/engine/session.rs

use llama_cpp_2::token::LlamaToken;

/// Holds mutable generation state that persists between requests.
///
/// Note: Model, context and batch are owned outside this struct to avoid
/// self‑referential lifetimes.
pub(crate) struct InferenceSession {
    /// Tokens of the last processed prompt plus generated tokens.
    pub history_tokens: Vec<LlamaToken>,
    /// Incomplete UTF‑8 byte sequence from the previous token decode.
    pub utf8_buffer: Vec<u8>,
    /// Current position in the sequence (number of tokens already processed).
    pub pos: i32,
}

impl InferenceSession {
    pub fn new() -> Self {
        Self {
            history_tokens: Vec::new(),
            utf8_buffer: Vec::with_capacity(32),
            pos: 0,
        }
    }

    pub fn reset(&mut self) {
        self.history_tokens.clear();
        self.utf8_buffer.clear();
        self.pos = 0;
    }
}
