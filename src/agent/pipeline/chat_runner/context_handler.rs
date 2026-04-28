// src/agent/pipeline/chat_runner/context_handler.rs

use super::StateManager;
use crate::agent::core::{Agent, EvictionHandler};
use crate::agent::tool::ToolDefinition;
use crate::error::Result;
use crate::llm::ChatTemplate;
use crate::types::config::EvictionStrategy;
use crate::types::message::{ContentPart, Message};
use crate::types::request::LLMRequest;

impl<'a> StateManager<'a> {
    pub async fn push_user_message(&self, parts: Vec<ContentPart>, tokens: usize) -> Result<()> {
        self.0
            .write()
            .await
            .chat_history
            .push(Message::User { content: parts }, tokens);
        Ok(())
    }

    pub async fn push_tool_message(
        &self,
        content: String,
        tool_id: Option<String>,
        tokens: usize,
    ) -> Result<()> {
        self.0
            .write()
            .await
            .chat_history
            .push(Message::Tool { content, tool_id }, tokens);
        Ok(())
    }

    pub async fn get_snapshot_len(&self) -> Result<usize> {
        Ok(self.0.read().await.chat_history.len())
    }

    pub async fn truncate(&self, len: usize) -> Result<()> {
        self.0.write().await.chat_history.truncate(len);
        Ok(())
    }

    pub async fn get_llm_request(
        &self,
        system_prompt: &str,
        tpl: &ChatTemplate,
        tools: &[ToolDefinition],
        cached_tool_prompt: &str,
        tool_tags: (String, String),
    ) -> Result<LLMRequest> {
        let lock = self.0.read().await;
        Ok(Agent::get_llm_request(
            &lock,
            system_prompt,
            tpl,
            tools,
            cached_tool_prompt,
            tool_tags,
        ))
    }

    pub async fn get_system_overhead(&self) -> Result<usize> {
        Ok(self
            .0
            .read()
            .await
            .chat_history
            .all()
            .iter()
            .filter(|(m, _)| matches!(**m, Message::System { .. }))
            .map(|(_, t)| *t)
            .sum())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn append_assistant_message_and_evict(
        &self,
        content: String,
        tool_calls: Vec<(String, serde_json::Value, String)>,
        tokens: usize,
        handler: &Option<EvictionHandler>,
        eviction_strategy: &EvictionStrategy,
        prompt_overhead: usize,
    ) -> Result<usize> {
        let mut lock = self.0.write().await;
        lock.chat_history.push(
            Message::Assistant {
                content,
                tool_calls,
            },
            tokens,
        );

        let evicted_msgs = lock.chat_history.evict_old_messages(
            eviction_strategy.keep_head,
            eviction_strategy.keep_tail,
            eviction_strategy.max_safe_tokens,
            prompt_overhead,
        );

        let count = evicted_msgs.len();
        if count > 0 {
            log::debug!("Context truncation: Evicted {} messages.", count);
            if let Some(h) = handler {
                h(evicted_msgs);
            }
        }
        Ok(count)
    }
}
