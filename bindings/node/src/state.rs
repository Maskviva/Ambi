// bindings/node/src/state.rs

use crate::agent::JsAgent;
use crate::message::{convert_message, JsMessage};
use ambi::AgentState;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::RwLock;

#[napi(js_name = "AgentState")]
#[derive(Clone)]
pub struct JsAgentState {
    pub(crate) inner: Arc<RwLock<AgentState>>,
}

#[napi]
impl JsAgentState {
    #[napi(constructor)]
    pub fn new(session_id: String) -> Self {
        Self {
            inner: AgentState::new_shared(session_id),
        }
    }

    #[napi(getter)]
    pub async fn session_id(&self) -> String {
        self.inner.read().await.session_id.clone()
    }

    #[napi]
    pub async fn get_dynamic_context(&self) -> Result<String> {
        Ok(self.inner.read().await.dynamic_context.clone())
    }

    #[napi]
    pub async fn set_dynamic_context(&self, context: String) -> Result<()> {
        self.inner.write().await.set_dynamic_context(&context);
        Ok(())
    }

    #[napi]
    pub async fn append_dynamic_context(&self, context: String) -> Result<()> {
        self.inner.write().await.append_dynamic_context(&context);
        Ok(())
    }

    #[napi]
    pub async fn clear_dynamic_context(&self) -> Result<()> {
        self.inner.write().await.clear_dynamic_context();
        Ok(())
    }

    #[napi]
    pub async fn clear_history(&self, agent: &JsAgent) -> Result<()> {
        let mut state = self.inner.write().await;
        state.clear_history(&agent.inner);
        Ok(())
    }

    #[napi]
    pub async fn history_len(&self) -> u32 {
        self.inner.read().await.chat_history.len() as u32
    }

    #[napi]
    pub async fn history_is_empty(&self) -> bool {
        self.inner.read().await.chat_history.is_empty()
    }

    #[napi]
    pub async fn history_all(&self) -> Vec<JsMessage> {
        self.inner
            .read()
            .await
            .chat_history
            .all()
            .iter()
            .map(|(msg, _)| convert_message(msg))
            .collect()
    }

    #[napi]
    pub async fn history_search_by_keyword(&self, keyword: String) -> Vec<JsMessage> {
        self.inner
            .read()
            .await
            .chat_history
            .search_by_keyword(&keyword)
            .iter()
            .map(|msg| convert_message(msg))
            .collect()
    }

    #[napi]
    pub async fn last_user_message(&self) -> Option<JsMessage> {
        self.inner
            .read()
            .await
            .chat_history
            .last_user_message()
            .map(|msg| convert_message(&msg))
    }

    #[napi]
    pub async fn last_assistant_message(&self) -> Option<JsMessage> {
        self.inner
            .read()
            .await
            .chat_history
            .last_assistant_message()
            .map(|msg| convert_message(&msg))
    }

    #[napi]
    pub async fn history_truncate(&self, len: u32) -> Result<()> {
        self.inner.write().await.chat_history.truncate(len as usize);
        Ok(())
    }

    #[napi]
    pub async fn history_total_tokens(&self) -> u32 {
        self.inner.read().await.chat_history.total_tokens() as u32
    }

    #[napi]
    pub async fn fork(&self) -> JsAgentState {
        let state = self.inner.read().await;
        JsAgentState {
            inner: state.fork_shared(),
        }
    }
}
