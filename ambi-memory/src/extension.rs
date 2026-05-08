// ambi-memory/src/extension.rs

use crate::error::{MemoryError, Result};
use crate::provider::kv::KvMemoryProvider;
use crate::provider::semantic::SemanticMemoryProvider;
use crate::provider::summary::SummaryMemoryProvider;

use ambi::agent::core::{Agent, AgentState};
use ambi::Message;
use async_trait::async_trait;
use std::sync::Arc;

/// The grand unified extension trait that empowers `AgentState` with 3D cognitive memory.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait AgentStateMemoryExt {
    // --- 1. KV State Memory (Reflexion / Settings) ---
    async fn remember_kv(
        &self,
        provider: &dyn KvMemoryProvider,
        key: &str,
        value: &str,
    ) -> Result<()>;
    async fn recall_kv_into_context(&mut self, provider: &dyn KvMemoryProvider) -> Result<()>;

    // --- 2. Semantic Vector Memory (Long-term interactions) ---
    async fn archive_semantic(
        &self,
        provider: &dyn SemanticMemoryProvider,
        text: &str,
    ) -> Result<()>;
    async fn recall_semantic_into_context(
        &mut self,
        provider: &dyn SemanticMemoryProvider,
        query: &str,
        limit: usize,
    ) -> Result<()>;

    // --- 3. Rolling Summary Memory (Anti-amnesia on eviction) ---
    async fn inject_summary_context(&mut self, provider: &dyn SummaryMemoryProvider) -> Result<()>;

    /// **Black Magic**: Automatically digests evicted messages into a rolling summary using the Agent itself!
    async fn summarize_evicted_messages(
        &self,
        agent: &Agent,
        provider: &dyn SummaryMemoryProvider,
        evicted: &[Arc<Message>],
    ) -> Result<()>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl AgentStateMemoryExt for AgentState {
    // --- KV Memory Implementations ---
    async fn remember_kv(
        &self,
        provider: &dyn KvMemoryProvider,
        key: &str,
        value: &str,
    ) -> Result<()> {
        provider.store(&self.session_id, key, value).await
    }

    async fn recall_kv_into_context(&mut self, provider: &dyn KvMemoryProvider) -> Result<()> {
        let memories = provider.retrieve_all(&self.session_id).await?;
        if !memories.is_empty() {
            let mut compiled = String::from("[STATE MEMORY]:\n");
            for (k, v) in memories {
                compiled.push_str(&format!("- {}: {}\n", k, v));
            }
            self.append_dynamic_context(&compiled);
        }
        Ok(())
    }

    // --- Semantic Vector Memory Implementations ---
    async fn archive_semantic(
        &self,
        provider: &dyn SemanticMemoryProvider,
        text: &str,
    ) -> Result<()> {
        provider.add_memory(&self.session_id, text).await
    }

    async fn recall_semantic_into_context(
        &mut self,
        provider: &dyn SemanticMemoryProvider,
        query: &str,
        limit: usize,
    ) -> Result<()> {
        let memories = provider
            .search_memories(&self.session_id, query, limit)
            .await?;
        if !memories.is_empty() {
            let mut compiled = String::from("[RECALLED PAST EXPERIENCES]:\n");
            for (i, m) in memories.iter().enumerate() {
                compiled.push_str(&format!("{}. {}\n", i + 1, m));
            }
            self.append_dynamic_context(&compiled);
        }
        Ok(())
    }

    // --- Rolling Summary Memory Implementations ---
    async fn inject_summary_context(&mut self, provider: &dyn SummaryMemoryProvider) -> Result<()> {
        if let Some(summary) = provider.get_summary(&self.session_id).await? {
            let formatted = format!("[CONVERSATION SUMMARY]:\n{}\n", summary);
            // We append this summary to the dynamic context so the LLM remembers the distant past
            self.append_dynamic_context(&formatted);
        }
        Ok(())
    }

    async fn summarize_evicted_messages(
        &self,
        agent: &Agent,
        provider: &dyn SummaryMemoryProvider,
        evicted: &[Arc<Message>],
    ) -> Result<()> {
        if evicted.is_empty() {
            return Ok(());
        }

        let old_summary = provider
            .get_summary(&self.session_id)
            .await?
            .unwrap_or_else(|| "No previous summary.".to_string());

        let mut evicted_text = String::new();
        for msg in evicted {
            evicted_text.push_str(&format!("{}\n", msg));
        }

        let instruction = format!(
            "You are an AI assistant helping to compress conversational memory.\n\
            [Current Summary]: {}\n\
            [New Evicted Messages]: {}\n\
            Task: Merge the new messages into the current summary. Keep it concise, capturing crucial facts, entities, and context. Output ONLY the new summary.",
            old_summary, evicted_text
        );

        // We use the agent to process the summarization out-of-band!
        let runner = ambi::ChatRunner::new(1); // Internal transient runner

        // Create an ephemeral, isolated state just for this summarization task
        // We do NOT want the summarization process to pollute the current user's chat history!
        let temp_state = AgentState::new_shared(format!("{}_summarizer", self.session_id));

        // Execute the summarization using the core LLM engine
        let new_summary = runner
            .chat(agent, &temp_state, &instruction)
            .await
            .map_err(|e| {
                MemoryError::SummaryError(format!("LLM failed to generate summary: {}", e))
            })?;

        // Update the persistent summary provider
        provider
            .update_summary(&self.session_id, &new_summary)
            .await?;

        log::info!(
            "Successfully compressed {} evicted messages into rolling summary.",
            evicted.len()
        );
        Ok(())
    }
}
