// bindings/node/src/engine.rs
use ambi::error::AmbiError;
use ambi::llm::LLMEngineTrait;
use ambi::types::LLMRequest;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

// ---------------------------------------------------------------------------
// Global pending request tracker: sends a request_id to JS, JS calls back
// ---------------------------------------------------------------------------
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn pending_requests() -> &'static Mutex<HashMap<String, oneshot::Sender<String>>> {
    static PENDING: std::sync::OnceLock<Mutex<HashMap<String, oneshot::Sender<String>>>> =
        std::sync::OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Called from JS when an async LLM callback finishes.
#[napi]
pub fn resolve_request(request_id: String, result: String) -> napi::Result<()> {
    let sender = pending_requests()
        .lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?
        .remove(&request_id)
        .ok_or_else(|| Error::from_reason(format!("Unknown request id: {}", request_id)))?;
    sender
        .send(result)
        .map_err(|_| Error::from_reason("Receiver dropped"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// JsEngineBridge
// ---------------------------------------------------------------------------
pub struct JsEngineBridge {
    pub chat_fn: ThreadsafeFunction<String>,
    pub chat_stream_fn: Option<ThreadsafeFunction<String>>,
    pub supports_vision: bool,
}

#[async_trait::async_trait]
impl LLMEngineTrait for JsEngineBridge {
    async fn chat(&self, request: LLMRequest) -> ambi::error::Result<String> {
        let request_id = NEXT_ID.fetch_add(1, Ordering::Relaxed).to_string();
        let payload = serde_json::json!({
            "request_id": request_id,
            "request": request,
        });
        let req_json = serde_json::to_string(&payload).unwrap();

        let (tx, rx) = oneshot::channel();
        pending_requests()
            .lock()
            .map_err(|e| AmbiError::EngineError(e.to_string()))?
            .insert(request_id, tx);

        self.chat_fn.call(Ok(req_json), ThreadsafeFunctionCallMode::NonBlocking);

        rx.await
            .map_err(|_| AmbiError::EngineError("JS callback channel closed".into()))
    }

    async fn chat_stream(&self, request: LLMRequest, tx: Sender<ambi::error::Result<String>>) {
        let _ = tx.send(self.chat(request).await).await;
    }

    fn reset_context(&self) {}

    fn supports_multimodal(&self) -> bool {
        self.supports_vision
    }
}
