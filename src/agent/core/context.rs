// src/agent/core/context.rs

use crate::types::message::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Default)]
pub struct ChatHistory {
    messages: Vec<(Arc<Message>, usize)>,
    cached_text_len: usize,
    total_tokens: usize,
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            cached_text_len: 0,
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
        self.cached_text_len = 0;
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

    pub fn estimate_tokens(&self) -> usize {
        if self.total_tokens > 0 {
            self.total_tokens
        } else {
            self.cached_text_len / 4
        }
    }

    fn recalculate_len(&mut self) {
        self.cached_text_len = self.messages.iter().map(|m| m.0.text_len()).sum();
        self.total_tokens = self.messages.iter().map(|(_, t)| *t).sum();
    }

    pub fn evict_old_messages(
        &mut self,
        initial_keep: usize,
        recent_keep: usize,
        max_safe_tokens: usize,
        prompt_overhead: usize,
    ) -> Vec<Arc<Message>> {
        let total = self.messages.len();
        let total_tokens = self.estimate_tokens();

        if total <= initial_keep + recent_keep
            && (total_tokens + prompt_overhead) <= max_safe_tokens
        {
            return Vec::new();
        }

        let mut safe_cut_idx = total.saturating_sub(recent_keep).max(initial_keep);

        let mut remaining_len: usize = self.messages[safe_cut_idx..]
            .iter()
            .map(|m| m.0.text_len())
            .sum();

        while safe_cut_idx < total {
            if (remaining_len / 4 + prompt_overhead) <= max_safe_tokens {
                break;
            }
            remaining_len = remaining_len.saturating_sub(self.messages[safe_cut_idx].0.text_len());
            safe_cut_idx += 1;
        }

        while safe_cut_idx < total && safe_cut_idx > initial_keep {
            if matches!(&*self.messages[safe_cut_idx].0, Message::User { .. }) {
                break;
            }
            safe_cut_idx += 1;
        }

        if safe_cut_idx >= total {
            safe_cut_idx = total.saturating_sub(1).max(initial_keep);
        }

        let evicted = self
            .messages
            .drain(initial_keep..safe_cut_idx)
            .map(|(m, _)| m)
            .collect::<Vec<_>>();

        if initial_keep > 0
            && self.messages.len() > initial_keep
            && matches!(&*self.messages[initial_keep - 1].0, Message::User { .. })
            && matches!(&*self.messages[initial_keep].0, Message::User { .. })
        {
            self.messages.insert(
                initial_keep,
                (
                    Arc::new(Message::Assistant {
                        content: "[...History Truncated...]".to_string(),
                        tool_calls: vec![],
                    }),
                    0,
                ),
            );
        }

        self.recalculate_len();

        evicted
    }
}
