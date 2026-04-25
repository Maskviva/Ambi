// src/agent/pipeline/chat_runner/context_handler.rs

use super::StateManager;
use crate::agent::core::{Agent, AgentState, EvictionHandler};
use crate::agent::tool::ToolDefinition;
use crate::error::{AmbiError, Result};
use crate::llm::ChatTemplate;
use crate::types::message::{ContentPart, Message};
use crate::types::request::LLMRequest;
use std::sync::MutexGuard;

impl<'a> StateManager<'a> {
    fn get_lock(&self) -> Result<MutexGuard<'_, AgentState>> {
        self.0.lock().map_err(|_| {
            AmbiError::AgentError(
                "AgentState lock is poisoned due to a previous panic.".to_string(),
            )
        })
    }

    pub fn push_user_message(&self, parts: Vec<ContentPart>) -> Result<()> {
        self.get_lock()?
            .chat_history
            .push(Message::User { content: parts });
        Ok(())
    }

    pub fn push_tool_message(&self, content: String) -> Result<()> {
        self.get_lock()?
            .chat_history
            .push(Message::Tool { content });
        Ok(())
    }

    pub fn get_snapshot_len(&self) -> Result<usize> {
        Ok(self.get_lock()?.chat_history.len())
    }

    pub fn truncate(&self, len: usize) -> Result<()> {
        self.get_lock()?.chat_history.truncate(len);
        Ok(())
    }

    pub fn get_llm_request(
        &self,
        system_prompt: &str,
        tpl: &ChatTemplate,
        tools: &[ToolDefinition],
        cached_tool_prompt: &str,
    ) -> Result<LLMRequest> {
        let lock = self.get_lock()?;
        Ok(Agent::get_llm_request(
            &lock,
            system_prompt,
            tpl,
            tools,
            cached_tool_prompt,
        ))
    }

    pub fn get_system_overhead(&self) -> Result<usize> {
        Ok(self
            .get_lock()?
            .chat_history
            .all()
            .iter()
            .filter(|m| matches!(***m, Message::System { .. }))
            .map(|m| m.estimate_tokens())
            .sum())
    }

    pub fn append_assistant_message_and_evict(
        &self,
        content: String,
        handler: &Option<EvictionHandler>,
        eviction_strategy: (usize, usize, usize),
        prompt_overhead: usize,
    ) -> Result<usize> {
        let mut lock = self.get_lock()?;
        lock.chat_history.push(Message::Assistant { content });

        let evicted_msgs = lock.chat_history.evict_old_messages(
            eviction_strategy.0,
            eviction_strategy.1,
            eviction_strategy.2,
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
