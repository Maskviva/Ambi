use crate::agent::JsAgent;
use crate::message::JsContentPart;
use crate::state::JsAgentState;
use ambi::agent::core::{Agent, AgentState};
use ambi::agent::pipeline::Pipeline as AmbiPipeline;
use ambi::error::AmbiError;
use ambi::error::Result as AmbiResult;
use ambi::{ChatRunner, ContentPart};
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::{Mutex, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

// ---------------------------------------------------------------------------
// Global pending requests for pipeline callbacks
// ---------------------------------------------------------------------------
static NEXT_PIPELINE_ID: AtomicU64 = AtomicU64::new(1);

fn pending_pipeline_requests() -> &'static std::sync::Mutex<HashMap<String, oneshot::Sender<String>>> {
    static PENDING: std::sync::OnceLock<std::sync::Mutex<HashMap<String, oneshot::Sender<String>>>> =
        std::sync::OnceLock::new();
    PENDING.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Called from JS when an async pipeline callback finishes.
#[napi]
pub fn resolve_pipeline_request(request_id: String, result: String) -> napi::Result<()> {
    let sender = pending_pipeline_requests()
        .lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?
        .remove(&request_id)
        .ok_or_else(|| Error::from_reason(format!("Unknown pipeline request id: {}", request_id)))?;
    sender
        .send(result)
        .map_err(|_| Error::from_reason("Receiver dropped"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// JsPipelineBridge
// ---------------------------------------------------------------------------
pub struct JsPipelineBridge {
    pub execute_fn: ThreadsafeFunction<String>,
    pub execute_stream_fn: Option<ThreadsafeFunction<String>>,
}

impl AmbiPipeline for JsPipelineBridge {
    async fn execute(
        &self,
        _agent: &Agent,
        _state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> AmbiResult<String> {
        let request_id = NEXT_PIPELINE_ID.fetch_add(1, Ordering::Relaxed).to_string();
        let payload = serde_json::json!({
            "request_id": request_id,
            "input": input,
        });
        let input_json = serde_json::to_string(&payload).unwrap();

        let (tx, rx) = oneshot::channel();
        pending_pipeline_requests()
            .lock()
            .map_err(|e| AmbiError::PipelineError(e.to_string()))?
            .insert(request_id, tx);

        self.execute_fn
            .call(Ok(input_json), ThreadsafeFunctionCallMode::NonBlocking);

        rx.await
            .map_err(|_| AmbiError::PipelineError("Pipeline callback channel closed".into()))
    }

    async fn execute_stream(
        &self,
        _agent: &Agent,
        _state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> AmbiResult<Pin<Box<ReceiverStream<AmbiResult<String>>>>> {
        let (tx, rx) = tokio::sync::mpsc::channel(16);

        let result = self.execute(_agent, _state, input).await;
        let _ = tx.send(result).await;
        drop(tx);

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

enum PipelineImpl {
    ChatRunner(ChatRunner),
    JsBridge(JsPipelineBridge),
}

impl AmbiPipeline for PipelineImpl {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> AmbiResult<String> {
        match self {
            PipelineImpl::ChatRunner(runner) => runner.execute(agent, state, input).await,
            PipelineImpl::JsBridge(bridge) => bridge.execute(agent, state, input).await,
        }
    }

    async fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> AmbiResult<Pin<Box<ReceiverStream<AmbiResult<String>>>>> {
        match self {
            PipelineImpl::ChatRunner(runner) => runner.execute_stream(agent, state, input).await,
            PipelineImpl::JsBridge(bridge) => bridge.execute_stream(agent, state, input).await,
        }
    }
}

#[napi(js_name = "Pipeline")]
#[derive(Clone)]
pub struct JsPipeline {
    inner: Arc<PipelineImpl>,
}

#[napi]
impl JsPipeline {
    #[napi(factory)]
    pub fn chat_runner(max_concurrency: Option<u32>) -> Self {
        Self {
            inner: Arc::new(PipelineImpl::ChatRunner(ChatRunner::new(
                max_concurrency.unwrap_or(5) as usize,
            ))),
        }
    }

    #[napi(
        factory,
        ts_args_type = "executeHandler: (_err: Error | null, argsJson: string) => void, streamHandler?: (_err: Error | null, argsJson: string) => void"
    )]
    pub fn custom(execute_handler: Function, stream_handler: Option<Function>) -> Result<Self> {
        let val = execute_handler.value();
        let tsfn: ThreadsafeFunction<String> =
            unsafe { FromNapiValue::from_napi_value(val.env, val.value)? };

        let stream_tsfn = if let Some(sh) = stream_handler {
            let val = sh.value();
            Some(unsafe { FromNapiValue::from_napi_value(val.env, val.value)? })
        } else {
            None
        };

        Ok(Self {
            inner: Arc::new(PipelineImpl::JsBridge(JsPipelineBridge {
                execute_fn: tsfn,
                execute_stream_fn: stream_tsfn,
            })),
        })
    }

    #[napi]
    pub async fn chat(
        &self,
        agent: &JsAgent,
        state: &JsAgentState,
        prompt: String,
    ) -> Result<String> {
        let parts = vec![ContentPart::Text { text: prompt }];
        self.inner
            .execute(&agent.inner, &state.inner, parts)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn execute(
        &self,
        agent: &JsAgent,
        state: &JsAgentState,
        input: Vec<Value>,
    ) -> Result<String> {
        let mut parts = Vec::new();
        for val in input {
            let part: ContentPart =
                serde_json::from_value(val).map_err(|e| Error::from_reason(e.to_string()))?;
            parts.push(part);
        }
        self.inner
            .execute(&agent.inner, &state.inner, parts)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn execute_parts(
        &self,
        agent: &JsAgent,
        state: &JsAgentState,
        input: Vec<JsContentPart>,
    ) -> Result<String> {
        let mut parts = Vec::new();
        for part in &input {
            if let Some(cp) = super::message::convert_content_part(part) {
                parts.push(cp);
            }
        }
        self.inner
            .execute(&agent.inner, &state.inner, parts)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn chat_stream(
        &self,
        agent: &JsAgent,
        state: &JsAgentState,
        prompt: String,
    ) -> Result<JsChatStream> {
        let parts = vec![ContentPart::Text { text: prompt }];
        let rx_stream = self
            .inner
            .execute_stream(&agent.inner, &state.inner, parts)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(JsChatStream {
            stream: Arc::new(Mutex::new(rx_stream)),
        })
    }

    #[napi]
    pub async fn clear_history(agent: &JsAgent, state: &JsAgentState) -> Result<()> {
        let mut state_lock = state.inner.write().await;
        ChatRunner::clear_history(&agent.inner, &mut state_lock);
        Ok(())
    }
}

type StreamInner = Pin<Box<ReceiverStream<AmbiResult<String>>>>;

#[napi(js_name = "ChatStream")]
pub struct JsChatStream {
    stream: Arc<Mutex<StreamInner>>,
}

#[napi]
impl JsChatStream {
    #[napi]
    pub async fn next_chunk(&self) -> Result<Option<String>> {
        let mut stream = self.stream.lock().await;
        if let Some(res) = stream.next().await {
            match res {
                Ok(token) => Ok(Some(token)),
                Err(e) => Err(Error::from_reason(e.to_string())),
            }
        } else {
            Ok(None)
        }
    }
}
