use crate::types::message::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Default)]
pub struct ChatHistory {
    messages: Vec<Arc<Message>>,
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, msg: Message) {
        self.messages.push(Arc::new(msg));
    }

    pub fn all(&self) -> &[Arc<Message>] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn truncate(&mut self, len: usize) {
        self.messages.truncate(len);
    }

    pub fn estimate_tokens(&self) -> usize {
        self.messages.iter().map(|m| m.text_len()).sum::<usize>() / 4
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

        while safe_cut_idx < total {
            let remaining_tokens: usize = self.messages[safe_cut_idx..]
                .iter()
                .map(|m| m.text_len())
                .sum::<usize>()
                / 4;

            if (remaining_tokens + prompt_overhead) <= max_safe_tokens {
                break;
            }
            safe_cut_idx += 1;
        }

        while safe_cut_idx < total && safe_cut_idx > initial_keep {
            if matches!(&*self.messages[safe_cut_idx], Message::User { .. }) {
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
            .collect::<Vec<_>>();

        if initial_keep > 0
            && self.messages.len() > initial_keep
            && matches!(&*self.messages[initial_keep - 1], Message::User { .. })
            && matches!(&*self.messages[initial_keep], Message::User { .. })
        {
            self.messages.insert(
                initial_keep,
                Arc::new(Message::Assistant {
                    content: "[...History Truncated...]".to_string(),
                }),
            );
        }

        evicted
    }
}
