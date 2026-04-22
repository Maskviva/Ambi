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

    pub fn evict_old_messages(
        &mut self,
        initial_keep: usize,
        recent_keep: usize,
    ) -> Vec<Arc<Message>> {
        let total = self.messages.len();
        if total <= initial_keep + recent_keep {
            return Vec::new();
        }
        let max_evict_idx = total - recent_keep;

        let mut safe_cut_idx = 0;
        for i in (initial_keep..=max_evict_idx).rev() {
            if matches!(&*self.messages[i], Message::User { .. }) {
                safe_cut_idx = i;
                break;
            }
        }

        if safe_cut_idx <= initial_keep {
            log::warn!("No User message found in eviction window, forcing hard truncate.");
            safe_cut_idx = max_evict_idx;
        }
        self.messages.drain(initial_keep..safe_cut_idx).collect()
    }
}
