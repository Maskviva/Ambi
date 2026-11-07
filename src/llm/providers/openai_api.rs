// src/llm/providers/openai_api.rs

pub mod config;
mod stream;
mod sync;
/// Request/response translators for OpenAI API compatibility.
pub mod translator;

use self::config::OpenAIEngineConfig;
use crate::error::{AmbiError, Result};
use crate::impl_as_any;
use crate::llm::LLMEngineTrait;
use crate::runtime::timeout;
use crate::types::LLMRequest;
use async_openai::config::OpenAIConfig;
use async_openai::Client;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// How many requests may be in flight by default.
///
/// A single `Agent` fans out tool calls concurrently, and one engine handle is shared by every
/// conversation using it. Without a ceiling, a burst turns into an unbounded number of sockets
/// and an unbounded amount of provider-side rate-limit pressure.
const DEFAULT_MAX_CONCURRENCY: usize = 32;

/// How long one request may take before it is abandoned.
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// The OpenAI API engine implementation.
/// Wraps the async-openai client and provides integration with the Ambi framework.
///
/// # Concurrency and timeouts
///
/// In-flight requests are capped by a semaphore, so callers that exceed the ceiling wait rather
/// than opening more connections. Every request is also bounded by a timeout; when it fires the
/// request future is dropped, which closes the response stream.
///
/// The backend itself is stateless: `session_id` is ignored here.
#[derive(Clone)]
pub struct OpenAIEngine {
    client: Client<OpenAIConfig>,
    cfg: OpenAIEngineConfig,
    permits: Arc<Semaphore>,
    request_timeout: Duration,
}

impl OpenAIEngine {
    /// Loads and initializes an OpenAI engine with the given configuration.
    pub fn load(openai_cfg: OpenAIEngineConfig) -> Result<Self> {
        let mut config = OpenAIConfig::new().with_api_key(openai_cfg.api_key.clone());
        config = config.with_api_base(&openai_cfg.base_url);
        let client = Client::with_config(config);
        Ok(Self {
            client,
            cfg: openai_cfg,
            permits: Arc::new(Semaphore::new(DEFAULT_MAX_CONCURRENCY)),
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
        })
    }

    /// Sets the maximum number of concurrent in-flight requests.
    ///
    /// A value of `0` is treated as `1`; the engine never allows a deadlock by having no permits.
    pub fn with_max_concurrency(mut self, max_concurrency: usize) -> Self {
        self.permits = Arc::new(Semaphore::new(max_concurrency.max(1)));
        self
    }

    /// Sets the per-request timeout.
    pub fn with_request_timeout(mut self, request_timeout: Duration) -> Self {
        self.request_timeout = request_timeout;
        self
    }

    /// Waits for a concurrency permit, returning an error only if the semaphore was closed.
    ///
    /// The permit is owned rather than borrowed so the request future stays `Send` and does not
    /// hold a borrow of `self` for its whole life.
    async fn acquire_permit(&self) -> Result<OwnedSemaphorePermit> {
        Arc::clone(&self.permits)
            .acquire_owned()
            .await
            .map_err(|_| AmbiError::engine("OpenAI engine request gate was closed".to_string()))
    }

    fn timeout_error(&self) -> AmbiError {
        AmbiError::engine_kind(
            crate::error::EngineErrorKind::Transport,
            format!(
                "OpenAI request timed out after {}s",
                self.request_timeout.as_secs()
            ),
        )
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl LLMEngineTrait for OpenAIEngine {
    impl_as_any!();

    async fn chat(&self, request: LLMRequest) -> Result<String> {
        let _permit = self.acquire_permit().await?;

        match timeout(self.request_timeout, self.generate_response_sync(request)).await {
            Ok(result) => result,
            Err(_) => Err(self.timeout_error()),
        }
    }

    async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>) {
        // The permit is held for the whole stream, not just until the headers arrive: an open
        // stream still occupies a connection and still counts against the provider's limits.
        let _permit = match self.acquire_permit().await {
            Ok(permit) => permit,
            Err(e) => {
                let _ = tx.send(Err(e)).await;
                return;
            }
        };

        let result = timeout(
            self.request_timeout,
            self.generate_response_stream(request, tx.clone()),
        )
        .await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                let _ = tx.send(Err(e)).await;
            }
            Err(_) => {
                let _ = tx.send(Err(self.timeout_error())).await;
            }
        }
    }

    fn reset_context(&self) {
        // Stateless backend: there is no server-side conversation state to clear.
    }

    fn supports_multimodal(&self) -> bool {
        // The engine serves any OpenAI-compatible endpoint, most of which are text-only, so vision
        // cannot be assumed from the protocol. It is opt-in: either the caller names a model known
        // to accept images, or they say so explicitly with `OpenAIEngineConfig::with_vision`.
        self.cfg.supports_vision
    }
}
