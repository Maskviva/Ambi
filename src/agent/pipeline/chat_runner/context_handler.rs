// src/agent/pipeline/chat_runner/context_handler.rs
use super::ChatRunner;
use crate::agent::core::{CompletionRequest, EvictionHandler};
use crate::types::message::Message;
use tokio::sync::Mutex as TokioMutex;

impl ChatRunner {
    pub(crate) async fn append_user_message(
        req_mutex: &TokioMutex<CompletionRequest>,
        prompt: &str,
    ) {
        req_mutex
            .lock()
            .await
            .chat_history
            .push(Message::user_text(prompt));
    }

    pub(crate) async fn append_assistant_message_and_evict(
        req_mutex: &TokioMutex<CompletionRequest>,
        content: String,
        handler: &Option<EvictionHandler>,
        eviction_strategy: (usize, usize, usize),
        prompt_overhead: usize,
    ) -> usize {
        let evicted_msgs = {
            let mut req = req_mutex.lock().await;
            req.chat_history.push(Message::Assistant { content });
            req.chat_history.evict_old_messages(
                eviction_strategy.0,
                eviction_strategy.1,
                eviction_strategy.2,
                prompt_overhead,
            )
        };
        let count = evicted_msgs.len();
        if count > 0 {
            log::debug!("Context truncation: Evicted {} messages.", count);
            if let Some(h) = handler {
                h(evicted_msgs);
            }
        }
        count
    }
}
