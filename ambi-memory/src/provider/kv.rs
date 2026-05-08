//! Exact key-value memory provider for reflexion diaries, settings, and state flags.

use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Persistent key-value memory interface.
///
/// Each value is scoped to a `session_id`, allowing independent memory
/// namespaces per conversation.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait KvMemoryProvider: Send + Sync {
    /// Store a value under the given key for a session.
    async fn store(&self, session_id: &str, key: &str, value: &str) -> Result<()>;
    /// Retrieve a single value by key.
    async fn retrieve(&self, session_id: &str, key: &str) -> Result<Option<String>>;
    /// Retrieve all key-value pairs for a session.
    async fn retrieve_all(&self, session_id: &str) -> Result<HashMap<String, String>>;
    /// Remove all data associated with a session.
    async fn clear_session(&self, session_id: &str) -> Result<()>;
}

/// In-memory implementation of `KvMemoryProvider` (for testing and single-node deployments).
#[derive(Clone, Default)]
pub struct InMemoryKvProvider {
    store: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

impl InMemoryKvProvider {
    /// Creates an empty in-memory KV store.
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl KvMemoryProvider for InMemoryKvProvider {
    async fn store(&self, session_id: &str, key: &str, value: &str) -> Result<()> {
        let mut lock = self.store.write().await;
        lock.entry(session_id.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }
    async fn retrieve(&self, session_id: &str, key: &str) -> Result<Option<String>> {
        Ok(self
            .store
            .read()
            .await
            .get(session_id)
            .and_then(|s| s.get(key).cloned()))
    }
    async fn retrieve_all(&self, session_id: &str) -> Result<HashMap<String, String>> {
        Ok(self
            .store
            .read()
            .await
            .get(session_id)
            .cloned()
            .unwrap_or_default())
    }
    async fn clear_session(&self, session_id: &str) -> Result<()> {
        self.store.write().await.remove(session_id);
        Ok(())
    }
}
