// bindings/node/src/agent.rs

use super::config::JsChatTemplate;
use super::config::JsChatTemplateType;
use super::config::JsEvictionStrategy;
use super::engine::JsEngine;
use super::types::JsMessage;
use ambi::AgentState;
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::RwLock;

// ── AgentState ──

/// The mutable half of a conversation. `AgentState` is deliberately
/// decoupled from the `Agent` blueprint so a single agent can juggle
/// many independent conversations — one state per chat session.
#[napi]
pub struct JsAgentState {
    pub(crate) inner: Arc<RwLock<AgentState>>,
}

#[napi]
impl JsAgentState {
    /// Start with a blank slate.
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(AgentState::new())),
        }
    }

    /// Wipe the conversation history.
    #[napi]
    pub async fn clear_history(&self) {
        let mut state = self.inner.write().await;
        state.chat_history.clear();
    }

    /// How many messages are in the history right now.
    #[napi]
    pub async fn get_history_length(&self) -> u32 {
        let state = self.inner.read().await;
        state.chat_history.len() as u32
    }

    /// Retrieve the full history as an array of `Message` objects.
    #[napi]
    pub async fn get_history(&self) -> Vec<JsMessage> {
        let state = self.inner.read().await;
        state
            .chat_history
            .all()
            .iter()
            .map(|(msg, _)| JsMessage::from(msg.as_ref()))
            .collect()
    }
}

// ── Agent ──

/// The immutable blueprint — engine, tools, config, hooks.
///
/// Built with a fluent chain that mirrors the Rust builder:
///
/// ```js
/// const agent = await Agent.make(engine)
///   .preamble("You are a helpful assistant.")
///   .withStandardFormatting()
///   .withEvictionStrategy({ maxSafeTokens: 4096 });
/// ```
///
/// Each builder step clones the underlying `Agent` (cheap, since all
/// mutable parts are `Arc`-wrapped), so chaining never mutates the
/// previous instance.
#[napi]
pub struct JsAgent {
    pub(crate) inner: ambi::Agent,
}

#[napi]
impl JsAgent {
    // ── Factory ──

    /// Create an Agent from an already-constructed `Engine`.
    #[napi(factory)]
    pub async fn make(engine: &JsEngine) -> napi::Result<Self> {
        let agent = ambi::Agent::from_engine(Arc::clone(&engine.inner));
        Ok(Self { inner: agent })
    }

    // ── Builder: System Prompt ──

    /// Set the system prompt (preamble) for this agent.
    #[napi]
    pub fn preamble(&self, text: String) -> Self {
        Self {
            inner: self.inner.clone().preamble(&text),
        }
    }

    // ── Builder: Chat Template ──

    /// Pick a template by its well-known name.
    #[napi]
    pub fn set_template(&self, template_type: JsChatTemplateType) -> Self {
        let ty: ambi::types::ChatTemplateType = template_type.into();
        Self {
            inner: self.inner.clone().template(ty),
        }
    }

    /// Supply a fully custom template — every prefix and suffix, your way.
    #[napi]
    pub fn set_custom_template(&self, template: JsChatTemplate) -> Self {
        let inner = self.inner.clone().template(template);
        Self { inner }
    }

    // ── Builder: Eviction ──

    /// Tune how aggressively old messages get evicted when the token
    /// budget runs low.
    #[napi]
    pub fn with_eviction_strategy(&self, strategy: JsEvictionStrategy) -> Self {
        Self {
            inner: self.inner.clone().with_eviction_strategy(strategy.into()),
        }
    }

    // ── Builder: Formatter ──

    /// Apply the standard stream formatter, which strips tool-call
    /// syntax and renders think blocks cleanly in real time.
    #[napi]
    pub fn with_standard_formatting(&self) -> Self {
        Self {
            inner: self.inner.clone().with_standard_formatting(),
        }
    }

    // ── Capabilities ──

    /// Evaluate sentence entropy — only works with local engines that
    /// expose log-probabilities.
    #[napi]
    pub async fn evaluate_sentence_entropy(&self, sentence: String) -> napi::Result<f64> {
        self.inner
            .evaluate_sentence_entropy(&sentence)
            .await
            .map(|v| v as f64)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }
}
