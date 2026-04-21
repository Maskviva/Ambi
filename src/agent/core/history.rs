use crate::types::message::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct ChatHistory {
    messages: Vec<Message>,
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    pub fn all(&self) -> &[Message] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn evict_old_messages(&mut self, initial_keep: usize, recent_keep: usize) -> Vec<Message> {
        let total = self.messages.len();

        if total <= initial_keep + recent_keep {
            return Vec::new();
        }

        let max_evict_idx = total - recent_keep;

        let mut safe_cut_idx = 0;
        for i in (initial_keep..=max_evict_idx).rev() {
            if matches!(self.messages[i], Message::User { .. }) {
                safe_cut_idx = i;
                break;
            }
        }

        if safe_cut_idx <= initial_keep {
            return Vec::new();
        }

        self.messages.drain(initial_keep..safe_cut_idx).collect()
    }
}
