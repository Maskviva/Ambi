// src/agent/core/context.rs

use crate::types::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A structure storing dialogue turns and tracking aggregate token usage.
#[derive(Serialize, Deserialize, Default)]
pub struct ChatHistory {
    messages: Vec<(Arc<Message>, usize)>,
    total_tokens: usize,
}

impl ChatHistory {
    /// Initializes an empty chat history.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            total_tokens: 0,
        }
    }

    /// # Accessors
    pub fn all(&self) -> &[(Arc<Message>, usize)] {
        &self.messages
    }

    /// Returns the raw total token count currently held in history.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Checks if the chat history contains zero messages.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// # Modifiers
    pub fn push(&mut self, msg: Message, exact_tokens: usize) {
        self.total_tokens += exact_tokens;
        self.messages.push((Arc::new(msg), exact_tokens));
    }

    /// Clears the history completely and resets the token counter to zero.
    pub fn clear(&mut self) {
        self.messages.clear();
        self.total_tokens = 0;
    }

    /// Truncates the message list to the specified length and recalculates tokens.
    pub fn truncate(&mut self, len: usize) {
        if len < self.messages.len() {
            self.messages.truncate(len);
            self.recalculate_len();
        }
    }

    /// The robust Context Eviction algorithm. Pops the oldest messages until safe.
    pub fn evict_old_messages(
        &mut self,
        max_safe_tokens: usize, // Maximum allowed number of tokens (excluding prompt overhead)
        prompt_overhead: usize, // Number of tokens for system prompts and other fixed overhead
    ) -> Vec<Arc<Message>> {
        let mut target_tokens = self.total_tokens + prompt_overhead;

        if target_tokens <= max_safe_tokens || self.messages.is_empty() {
            return Vec::new();
        }

        let mut evict_count = 0;
        let mut tokens_to_remove = 0;

        for (_, tokens) in &self.messages {
            evict_count += 1;
            tokens_to_remove += tokens;
            target_tokens -= tokens;

            if target_tokens <= max_safe_tokens {
                break;
            }
        }

        let evicted: Vec<Arc<Message>> = self
            .messages
            .drain(0..evict_count)
            .map(|(m, _)| m)
            .collect();

        self.total_tokens -= tokens_to_remove;

        if !evicted.is_empty() {
            log::warn!(
                "Token limit exceeded (Max: {}). Evicted {} oldest messages.",
                max_safe_tokens,
                evicted.len()
            );
        }

        evicted
    }

    /// # Internal Helpers
    fn recalculate_len(&mut self) {
        self.total_tokens = self.messages.iter().map(|(_, t)| *t).sum();
    }
}
