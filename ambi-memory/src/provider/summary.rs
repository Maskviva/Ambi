//! Rolling summary memory provider for anti-amnesia on context eviction.
//!
//! When the FIFO eviction algorithm drops old conversation turns, this provider
//! persists compressed summaries so the agent can reconstruct distant context.

use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Rolling summary memory interface.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait SummaryMemoryProvider: Send + Sync {
    /// Overwrite the rolling summary for a session with a new compressed version.
    async fn update_summary(&self, session_id: &str, new_summary: &str) -> Result<()>;

    /// Retrieve the currently held rolling summary for a session.
    async fn get_summary(&self, session_id: &str) -> Result<Option<String>>;
}

/// In-memory implementation of `SummaryMemoryProvider` (for testing and single-node deployments).
#[derive(Clone, Default)]
pub struct InMemorySummaryProvider {
    store: Arc<RwLock<HashMap<String, String>>>,
}

impl InMemorySummaryProvider {
    /// Creates an empty in-memory summary store.
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl SummaryMemoryProvider for InMemorySummaryProvider {
    async fn update_summary(&self, session_id: &str, new_summary: &str) -> Result<()> {
        self.store
            .write()
            .await
            .insert(session_id.to_string(), new_summary.to_string());
        Ok(())
    }
    async fn get_summary(&self, session_id: &str) -> Result<Option<String>> {
        Ok(self.store.read().await.get(session_id).cloned())
    }
}
