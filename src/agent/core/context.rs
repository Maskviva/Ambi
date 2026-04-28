// src/agent/core/context.rs

use crate::types::message::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Default)]
pub struct ChatHistory {
    messages: Vec<(Arc<Message>, usize)>,
    total_tokens: usize,
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            total_tokens: 0,
        }
    }

    pub fn push(&mut self, msg: Message, exact_tokens: usize) {
        self.total_tokens += exact_tokens;
        self.messages.push((Arc::new(msg), exact_tokens));
    }

    pub fn all(&self) -> &[(Arc<Message>, usize)] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.total_tokens = 0;
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn truncate(&mut self, len: usize) {
        if len < self.messages.len() {
            self.messages.truncate(len);
            self.recalculate_len();
        }
    }

    fn recalculate_len(&mut self) {
        self.total_tokens = self.messages.iter().map(|(_, t)| *t).sum();
    }

    pub fn evict_old_messages(
        &mut self,
        initial_keep: usize,    // Keep the number of initial messages
        recent_keep: usize,     // Keep the number of recent messages
        max_safe_tokens: usize, // Maximum allowed number of tokens (excluding prompt overhead)
        prompt_overhead: usize, // Number of tokens for system prompts and other fixed overhead
    ) -> Vec<Arc<Message>> {
        let total = self.messages.len();
        if total <= initial_keep + recent_keep {
            return Vec::new();
        }

        // Calculate the total number of tokens for all current messages
        let total_tokens: usize = self.messages.iter().map(|(_, t)| *t).sum();

        // If it does not exceed the limit, eviction is not necessary.
        if total_tokens + prompt_overhead <= max_safe_tokens {
            return Vec::new();
        }

        // Eviction required: remove all messages between [initial_keep .. total - recent_keep)
        let start_remove = initial_keep;
        let end_remove = total - recent_keep;

        // If this range is empty, forcibly remove some old messages to avoid overflow (keep at least one)
        if start_remove >= end_remove {
            // Cannot meet the requirement, can only reduce the amount retained at the head or tail, here the tail is prioritized for reduction
            let new_recent = total.saturating_sub(start_remove + 1);
            return self.evict_old_messages(
                initial_keep,
                new_recent.max(1),
                max_safe_tokens,
                prompt_overhead,
            );
        }

        let evicted: Vec<_> = self
            .messages
            .drain(start_remove..end_remove)
            .map(|(m, _)| m)
            .collect();

        // Recalculate cache
        self.recalculate_len();

        evicted
    }
}
