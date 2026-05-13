use ambi::agent::core::{Agent, AgentState};
use ambi::agent::pipeline::Pipeline as AmbiPipeline;
use ambi::config::EvictionStrategy;
use ambi::error::AmbiError;
use ambi::llm::LLMEngineTrait;
use ambi::types::{
    ChatTemplate, ChatTemplateType, ContentPart, LLMRequest, Tool, ToolDefinition, ToolErr,
};
use ambi::{impl_as_any, ChatRunner, LLMEngineConfig};
use async_trait::async_trait;
use futures::StreamExt;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tokio_stream::wrappers::ReceiverStream;

// ============================================================
// Pending request tracker
// ============================================================
static NEXT_REQ_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_PIPELINE_ID: AtomicU64 = AtomicU64::new(1);

fn pending_reqs() -> &'static Mutex<HashMap<String, oneshot::Sender<String>>> {
    static P: std::sync::OnceLock<Mutex<HashMap<String, oneshot::Sender<String>>>> =
        std::sync::OnceLock::new();
    P.get_or_init(|| Mutex::new(HashMap::new()))
}

fn pending_pipeline_reqs() -> &'static Mutex<HashMap<String, oneshot::Sender<String>>> {
    static P: std::sync::OnceLock<Mutex<HashMap<String, oneshot::Sender<String>>>> =
        std::sync::OnceLock::new();
    P.get_or_init(|| Mutex::new(HashMap::new()))
}

#[pyfunction]
fn resolve_request(request_id: String, result: String) -> PyResult<()> {
    let sender = pending_reqs()
        .lock()
        .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?
        .remove(&request_id)
        .ok_or_else(|| PyRuntimeError::new_err(format!("Unknown request id: {}", request_id)))?;
    sender
        .send(result)
        .map_err(|_| PyRuntimeError::new_err("Receiver dropped"))?;
    Ok(())
}

#[pyfunction]
fn resolve_pipeline_request(request_id: String, result: String) -> PyResult<()> {
    let sender = pending_pipeline_reqs()
        .lock()
        .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?
        .remove(&request_id)
        .ok_or_else(|| {
            PyRuntimeError::new_err(format!("Unknown pipeline request id: {}", request_id))
        })?;
    sender
        .send(result)
        .map_err(|_| PyRuntimeError::new_err("Receiver dropped"))?;
    Ok(())
}

// ============================================================
// Template type constants (mirrors JS ChatTemplateType enum)
// ============================================================
fn parse_template_type(s: &str) -> PyResult<ChatTemplateType> {
    match s {
        "chatml" => Ok(ChatTemplateType::Chatml),
        "llama3" => Ok(ChatTemplateType::Llama3),
        "gemma" => Ok(ChatTemplateType::Gemma),
        "phi3" => Ok(ChatTemplateType::Phi3),
        "zephyr" => Ok(ChatTemplateType::Zephyr),
        "deepseek" => Ok(ChatTemplateType::Deepseek),
        "qwen" => Ok(ChatTemplateType::Qwen),
        "mistral" => Ok(ChatTemplateType::Mistral),
        "llama2" => Ok(ChatTemplateType::Llama2),
        _ => Err(PyValueError::new_err(format!(
            "Unknown template type: '{}'. Options: chatml, llama3, gemma, phi3, zephyr, deepseek, qwen, mistral, llama2",
            s
        ))),
    }
}

// ============================================================
// PyTool
// ============================================================
struct PyTool {
    name: String,
    description: String,
    parameters: Value,
    callback: PyObject,
    timeout_secs: Option<u64>,
    max_retries: Option<usize>,
    is_idempotent: bool,
}

unsafe impl Send for PyTool {}
unsafe impl Sync for PyTool {}

#[async_trait]
impl Tool for PyTool {
    const NAME: &'static str = "PYTHON_TOOL";
    type Args = Value;
    type Output = Value;

    fn name(&self) -> String {
        self.name.clone()
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.parameters.clone(),
            timeout_secs: self.timeout_secs,
            max_retries: self.max_retries,
            is_idempotent: self.is_idempotent,
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr> {
        let args_json =
            serde_json::to_string(&args).map_err(|e| ToolErr(format!("Serialize args: {}", e)))?;
        let result_str: String = Python::with_gil(|py| -> PyResult<String> {
            self.callback
                .call1(py, (args_json,))
                .and_then(|r| r.extract::<String>(py))
        })
        .map_err(|e| ToolErr(format!("Python callback error: {}", e)))?;
        serde_json::from_str(&result_str)
            .map_err(|e| ToolErr(format!("Invalid JSON result: {}", e)))
    }
}

// ============================================================
// PyEngineBridge
// ============================================================
struct PyEngineBridge {
    chat_handler: PyObject,
    supports_vision: bool,
    _stream_handler: Option<PyObject>,
}

unsafe impl Send for PyEngineBridge {}
unsafe impl Sync for PyEngineBridge {}

#[async_trait]
impl LLMEngineTrait for PyEngineBridge {
    impl_as_any!();

    async fn chat(&self, request: LLMRequest) -> Result<String, AmbiError> {
        let request_id = NEXT_REQ_ID.fetch_add(1, Ordering::Relaxed).to_string();
        let payload = serde_json::json!({ "request_id": request_id, "request": request });
        let req_json = serde_json::to_string(&payload).unwrap();

        let (tx, rx) = oneshot::channel();
        pending_reqs()
            .lock()
            .map_err(|e| AmbiError::EngineError(e.to_string()))?
            .insert(request_id, tx);

        Python::with_gil(|py| -> PyResult<()> {
            self.chat_handler.call1(py, (req_json,))?;
            Ok(())
        })
        .map_err(|e| AmbiError::EngineError(format!("Python handler error: {}", e)))?;

        rx.await
            .map_err(|_| AmbiError::EngineError("Python handler channel closed".into()))
    }

    async fn chat_stream(
        &self,
        request: LLMRequest,
        tx: tokio::sync::mpsc::Sender<Result<String, AmbiError>>,
    ) {
        let _ = tx.send(self.chat(request).await).await;
    }

    fn reset_context(&self) {}

    fn supports_multimodal(&self) -> bool {
        self.supports_vision
    }
}

// ============================================================
// PyPipelineBridge
// ============================================================
struct PyPipelineBridge {
    execute_handler: PyObject,
    _stream_handler: Option<PyObject>,
}

unsafe impl Send for PyPipelineBridge {}
unsafe impl Sync for PyPipelineBridge {}

impl AmbiPipeline for PyPipelineBridge {
    async fn execute(
        &self,
        _agent: &Agent,
        _state: &Arc<tokio::sync::RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String, AmbiError> {
        let request_id = NEXT_PIPELINE_ID.fetch_add(1, Ordering::Relaxed).to_string();
        let payload = serde_json::json!({ "request_id": request_id, "input": input });
        let input_json = serde_json::to_string(&payload).unwrap();

        let (tx, rx) = oneshot::channel();
        pending_pipeline_reqs()
            .lock()
            .map_err(|e| AmbiError::PipelineError(e.to_string()))?
            .insert(request_id, tx);

        Python::with_gil(|py| -> PyResult<()> {
            self.execute_handler.call1(py, (input_json,))?;
            Ok(())
        })
        .map_err(|e| AmbiError::PipelineError(format!("Python pipeline error: {}", e)))?;

        rx.await
            .map_err(|_| AmbiError::PipelineError("Pipeline channel closed".into()))
    }

    async fn execute_stream(
        &self,
        _agent: &Agent,
        _state: &Arc<tokio::sync::RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String, AmbiError>>>>, AmbiError> {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        let result = self.execute(_agent, _state, input).await;
        let _ = tx.send(result).await;
        drop(tx);
        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

// ============================================================
// PipelineImpl
// ============================================================
enum PipelineImpl {
    ChatRunner(ChatRunner),
    PyBridge(PyPipelineBridge),
}

impl AmbiPipeline for PipelineImpl {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<tokio::sync::RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String, AmbiError> {
        match self {
            PipelineImpl::ChatRunner(r) => r.execute(agent, state, input).await,
            PipelineImpl::PyBridge(b) => b.execute(agent, state, input).await,
        }
    }

    async fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<tokio::sync::RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String, AmbiError>>>>, AmbiError> {
        match self {
            PipelineImpl::ChatRunner(r) => r.execute_stream(agent, state, input).await,
            PipelineImpl::PyBridge(b) => b.execute_stream(agent, state, input).await,
        }
    }
}

// ============================================================
// Python: AgentState
// ============================================================
#[pyclass(name = "AgentState")]
#[derive(Clone)]
struct PyAgentState {
    inner: Arc<tokio::sync::RwLock<AgentState>>,
}

#[pymethods]
impl PyAgentState {
    #[new]
    fn new(session_id: String) -> Self {
        Self {
            inner: AgentState::new_shared(session_id),
        }
    }

    fn get_dynamic_context(&self) -> String {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async { self.inner.read().await.dynamic_context.clone() })
    }

    fn set_dynamic_context(&self, context: String) {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async { self.inner.write().await.set_dynamic_context(&context) });
    }

    fn clear_dynamic_context(&self) {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async { self.inner.write().await.clear_dynamic_context() });
    }

    fn append_dynamic_context(&self, context: String) {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async { self.inner.write().await.append_dynamic_context(&context) });
    }

    fn history_len(&self) -> u32 {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async { self.inner.read().await.chat_history.len() as u32 })
    }

    fn history_is_empty(&self) -> bool {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async { self.inner.read().await.chat_history.is_empty() })
    }

    fn fork(&self) -> Self {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async {
            let state = self.inner.read().await;
            PyAgentState {
                inner: state.fork_shared(),
            }
        })
    }
}

// ============================================================
// Python: LLMEngineConfig
// ============================================================
#[pyclass(name = "LLMEngineConfig")]
struct PyLLMEngineConfig {
    inner: Mutex<Option<LLMEngineConfig>>,
}

#[pymethods]
impl PyLLMEngineConfig {
    #[staticmethod]
    fn openai(
        api_key: String,
        model_name: String,
        base_url: Option<String>,
        temp: Option<f64>,
        top_p: Option<f64>,
    ) -> Self {
        use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
        let cfg = OpenAIEngineConfig {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model_name,
            temp: temp.unwrap_or(0.0) as f32,
            top_p: top_p.unwrap_or(0.0) as f32,
        };
        Self {
            inner: Mutex::new(Some(LLMEngineConfig::OpenAI(cfg))),
        }
    }

    #[staticmethod]
    fn custom(
        chat_handler: PyObject,
        supports_multimodal: Option<bool>,
        stream_handler: Option<PyObject>,
    ) -> Self {
        let bridge = Box::new(PyEngineBridge {
            chat_handler,
            supports_vision: supports_multimodal.unwrap_or(false),
            _stream_handler: stream_handler,
        });
        Self {
            inner: Mutex::new(Some(LLMEngineConfig::Custom(bridge))),
        }
    }
}

// ============================================================
// Python: Agent
// ============================================================
#[pyclass(name = "Agent")]
#[derive(Clone)]
struct PyAgent {
    inner: Agent,
}

#[pymethods]
impl PyAgent {
    #[staticmethod]
    fn make<'a>(py: Python<'a>, config: &'a PyLLMEngineConfig) -> PyResult<&'a PyAny> {
        let engine_cfg = {
            let mut lock = config
                .inner
                .lock()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            lock.take()
                .ok_or_else(|| PyRuntimeError::new_err("LLMEngineConfig can only be used once"))
        }?;

        let future = async move {
            let agent = Agent::make(engine_cfg)
                .await
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            Ok(PyAgent { inner: agent })
        };

        pyo3_asyncio::tokio::future_into_py(py, future)
    }

    fn preamble(&self, text: String) -> Self {
        Self {
            inner: self.inner.clone().preamble(&text),
        }
    }

    fn template(&self, template_type: String) -> PyResult<Self> {
        let ct = parse_template_type(&template_type)?;
        Ok(Self {
            inner: self.inner.clone().template(ct),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn custom_template(
        &self,
        system_prefix: String,
        system_suffix: String,
        user_prefix: String,
        user_suffix: String,
        assistant_prefix: String,
        assistant_suffix: String,
        think_prefix: String,
        think_suffix: String,
        tool_prefix: String,
        tool_suffix: String,
        tool_id_prefix: String,
        tool_id_suffix: String,
        media_placeholder: String,
    ) -> Self {
        let ct = ChatTemplate {
            system_prefix,
            system_suffix,
            user_prefix,
            user_suffix,
            assistant_prefix,
            assistant_suffix,
            think_prefix,
            think_suffix,
            tool_prefix,
            tool_suffix,
            tool_id_prefix,
            tool_id_suffix,
            media_placeholder,
        };
        Self {
            inner: self.inner.clone().template(ct),
        }
    }

    fn with_eviction_strategy(&self, max_safe_tokens: u32) -> Self {
        Self {
            inner: self.inner.clone().with_eviction_strategy(EvictionStrategy {
                max_safe_tokens: max_safe_tokens as usize,
            }),
        }
    }

    fn max_iterations(&self, n: u32) -> Self {
        Self {
            inner: self.inner.clone().max_iterations(n as usize),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_tool(
        &self,
        name: String,
        description: String,
        parameters_json: String,
        callback: PyObject,
        timeout_secs: Option<u64>,
        max_retries: Option<usize>,
        is_idempotent: Option<bool>,
    ) -> PyResult<Self> {
        let parameters: Value = serde_json::from_str(&parameters_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid parameters JSON: {}", e)))?;
        let py_tool = PyTool {
            name,
            description,
            parameters,
            callback,
            timeout_secs,
            max_retries,
            is_idempotent: is_idempotent.unwrap_or(true),
        };
        let inner = self
            .inner
            .clone()
            .tool(py_tool)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    fn with_standard_formatting(&self) -> Self {
        Self {
            inner: self.inner.clone().with_standard_formatting(),
        }
    }

    fn with_tool_tags(&self, start_tag: String, end_tag: String) -> Self {
        Self {
            inner: self.inner.clone().with_tool_tags(&start_tag, &end_tag),
        }
    }

    fn count_tokens(&self, text: String) -> PyResult<u32> {
        let engine = self.inner.get_llama_engine();
        engine
            .count_tokens(&text)
            .map(|n| n as u32)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

// ============================================================
// Python: ChatStream
// ============================================================
type StreamInner = Pin<Box<ReceiverStream<Result<String, AmbiError>>>>;

#[pyclass(name = "ChatStream")]
struct PyChatStream {
    stream: Arc<tokio::sync::Mutex<StreamInner>>,
}

#[pymethods]
impl PyChatStream {
    fn next_chunk<'p>(slf: PyRef<'p, Self>) -> PyResult<&'p PyAny> {
        let stream = slf.stream.clone();
        pyo3_asyncio::tokio::future_into_py(slf.py(), async move {
            let mut guard = stream.lock().await;
            match guard.next().await {
                Some(Ok(token)) => Ok(Some(token)),
                Some(Err(e)) => Err(PyRuntimeError::new_err(e.to_string())),
                None => Ok(None),
            }
        })
    }
}

// ============================================================
// Python: Pipeline
// ============================================================
#[pyclass(name = "Pipeline")]
#[derive(Clone)]
struct PyPipeline {
    inner: Arc<PipelineImpl>,
}

#[pymethods]
impl PyPipeline {
    #[staticmethod]
    fn chat_runner(max_concurrency: Option<u32>) -> Self {
        Self {
            inner: Arc::new(PipelineImpl::ChatRunner(ChatRunner::new(
                max_concurrency.unwrap_or(5) as usize,
            ))),
        }
    }

    #[staticmethod]
    fn custom(execute_handler: PyObject, stream_handler: Option<PyObject>) -> Self {
        Self {
            inner: Arc::new(PipelineImpl::PyBridge(PyPipelineBridge {
                execute_handler,
                _stream_handler: stream_handler,
            })),
        }
    }

    fn chat<'a>(
        &self,
        py: Python<'a>,
        agent: &PyAgent,
        state: &PyAgentState,
        prompt: String,
    ) -> PyResult<&'a PyAny> {
        let (inner, agent_inner, state_inner) =
            (self.inner.clone(), agent.inner.clone(), state.inner.clone());
        let parts = vec![ContentPart::Text { text: prompt }];

        pyo3_asyncio::tokio::future_into_py(py, async move {
            inner
                .execute(&agent_inner, &state_inner, parts)
                .await
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        })
    }

    fn chat_stream<'a>(
        &self,
        py: Python<'a>,
        agent: &PyAgent,
        state: &PyAgentState,
        prompt: String,
    ) -> PyResult<&'a PyAny> {
        let (inner, agent_inner, state_inner) =
            (self.inner.clone(), agent.inner.clone(), state.inner.clone());
        let parts = vec![ContentPart::Text { text: prompt }];

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let stream = inner
                .execute_stream(&agent_inner, &state_inner, parts)
                .await
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            Ok(PyChatStream {
                stream: Arc::new(tokio::sync::Mutex::new(stream)),
            })
        })
    }

    #[staticmethod]
    fn clear_history(agent: &PyAgent, state: &PyAgentState) {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async {
            let mut state_lock = state.inner.write().await;
            ChatRunner::clear_history(&agent.inner, &mut state_lock);
        });
    }
}

// ============================================================
// Template factory functions
// ============================================================
macro_rules! template_fn {
    ($name:ident, $method:ident) => {
        #[pyfunction]
        fn $name() -> HashMap<String, String> {
            let t = ChatTemplate::$method();
            let mut m = HashMap::new();
            m.insert("system_prefix".into(), t.system_prefix);
            m.insert("system_suffix".into(), t.system_suffix);
            m.insert("user_prefix".into(), t.user_prefix);
            m.insert("user_suffix".into(), t.user_suffix);
            m.insert("assistant_prefix".into(), t.assistant_prefix);
            m.insert("assistant_suffix".into(), t.assistant_suffix);
            m.insert("think_prefix".into(), t.think_prefix);
            m.insert("think_suffix".into(), t.think_suffix);
            m.insert("tool_prefix".into(), t.tool_prefix);
            m.insert("tool_suffix".into(), t.tool_suffix);
            m.insert("tool_id_prefix".into(), t.tool_id_prefix);
            m.insert("tool_id_suffix".into(), t.tool_id_suffix);
            m.insert("media_placeholder".into(), t.media_placeholder);
            m
        }
    };
}

template_fn!(chatml_template, chatml);
template_fn!(llama3_template, llama3);
template_fn!(gemma_template, gemma);
template_fn!(phi3_template, phi3);
template_fn!(zephyr_template, zephyr);
template_fn!(deepseek_template, deepseek);
template_fn!(qwen_template, qwen);
template_fn!(mistral_template, mistral);
template_fn!(llama2_template, llama2);

// Tool helpers are defined in pure Python (see python/ambi/__init__.py).
// They build JSON schemas and call agent.add_tool() with the right args.

// ============================================================
// Module definition
// ============================================================
#[pymodule]
fn ambi_python(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyAgent>()?;
    m.add_class::<PyAgentState>()?;
    m.add_class::<PyLLMEngineConfig>()?;
    m.add_class::<PyPipeline>()?;
    m.add_class::<PyChatStream>()?;

    m.add_function(wrap_pyfunction!(resolve_request, m)?)?;
    m.add_function(wrap_pyfunction!(resolve_pipeline_request, m)?)?;
    m.add_function(wrap_pyfunction!(chatml_template, m)?)?;
    m.add_function(wrap_pyfunction!(llama3_template, m)?)?;
    m.add_function(wrap_pyfunction!(gemma_template, m)?)?;
    m.add_function(wrap_pyfunction!(phi3_template, m)?)?;
    m.add_function(wrap_pyfunction!(zephyr_template, m)?)?;
    m.add_function(wrap_pyfunction!(deepseek_template, m)?)?;
    m.add_function(wrap_pyfunction!(qwen_template, m)?)?;
    m.add_function(wrap_pyfunction!(mistral_template, m)?)?;
    m.add_function(wrap_pyfunction!(llama2_template, m)?)?;

    Ok(())
}
