use crate::agent::message::Message;
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

    pub fn evict_for_memory(&mut self, initial_keep: usize, recent_keep: usize) -> Vec<Message> {
        let total = self.messages.len();

        if total <= initial_keep + recent_keep {
            return Vec::new();
        }

        let mut evict_count = total - initial_keep - recent_keep;

        if evict_count % 2 != 0 {
            evict_count -= 1;
        }

        if evict_count == 0 {
            return Vec::new();
        }

        let evicted = self
            .messages
            .drain(initial_keep..(initial_keep + evict_count))
            .collect();
        evicted
    }
}
