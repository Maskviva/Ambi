// bindings/node/src/engine.rs

use super::config::JsOpenAIConfig;
use super::types::JsLlmRequest;
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::llm::tokenizer::{DefaultTokenizer, TokenizerTrait};
use ambi::llm::{LLMEngine, LLMEngineConfig};
use napi::bindgen_prelude::Function;
use napi::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::sync::Arc;

// ── Helpers ──

/// Build a ThreadsafeFunction whose JS callback receives the bare value.
///
/// The function boundary breaks Rust's lifetime tracking: `Function<'_>`
/// has a non-static lifetime, but the TSFN only copies raw env/value
/// pointers and never retains the borrow. We erase the lifetime through
/// a helper so the caller can move the TSFN into a `tokio::spawn`.
fn make_tsfn_string(
    f: &Function<'_, String, ()>,
) -> napi::Result<
    napi::threadsafe_function::ThreadsafeFunction<
        String,
        (),
        String,
        napi::Status,
        false,
        false,
        0,
    >,
> {
    f.build_threadsafe_function::<String>()
        .build_callback(|ctx: ThreadsafeCallContext<String>| Ok(ctx.value))
}

fn make_tsfn_unit(
    f: &Function<'_, (), ()>,
) -> napi::Result<
    napi::threadsafe_function::ThreadsafeFunction<(), (), (), napi::Status, false, false, 0>,
> {
    f.build_threadsafe_function::<()>()
        .build_callback(|_ctx: ThreadsafeCallContext<()>| Ok(()))
}

// ── Tokenizer ──

/// A fast, synchronous token counter based on the `cl100k_base` BPE encoding.
/// Accurate enough for budget estimation without pulling in the full model
/// tokenizer on every request.
#[napi]
pub struct JsTokenizer {
    inner: DefaultTokenizer,
}

#[napi]
impl JsTokenizer {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        let tokenizer =
            DefaultTokenizer::make().map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(Self { inner: tokenizer })
    }

    #[napi]
    pub fn count_tokens(&self, text: String) -> napi::Result<u32> {
        self.inner
            .count_tokens(&text)
            .map(|c| c as u32)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }
}

// ── Engine ──

/// The LLM inference engine — wraps model backends behind a uniform API.
///
/// ```js
/// const engine = Engine.createOpenai(config);
/// await engine.chat(request);
/// engine.chatStream(request, onToken, onComplete, onError);
/// engine.resetContext();
/// ```
#[napi]
pub struct JsEngine {
    pub(crate) inner: Arc<LLMEngine>,
}

#[napi]
impl JsEngine {
    // ── Factory ──

    /// Build an engine backed by an OpenAI-compatible API endpoint.
    #[napi(factory)]
    pub fn create_openai(config: JsOpenAIConfig) -> napi::Result<Self> {
        let cfg = LLMEngineConfig::OpenAI(OpenAIEngineConfig {
            api_key: config.api_key,
            base_url: config.base_url,
            model_name: config.model_name,
            temp: config.temp as f32,
            top_p: config.top_p as f32,
        });
        let engine = LLMEngine::load(cfg).map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(engine),
        })
    }

    // ── Chat ──

    /// Send a fully-formed request to the LLM and collect the complete
    /// response. Useful for one-shot queries or custom pipelines that
    /// bypass the Agent loop.
    #[napi]
    pub async fn chat(&self, request: JsLlmRequest) -> napi::Result<String> {
        let rust_req: ambi::types::LLMRequest = request
            .try_into()
            .map_err(|e: String| napi::Error::from_reason(e))?;
        self.inner
            .chat(rust_req)
            .await
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Stream the LLM response token by token. The stream runs in the
    /// background once this function returns — callbacks deliver tokens
    /// as they arrive.
    ///
    /// ```js
    /// engine.chatStream(
    ///   request,
    ///   (token) => process.stdout.write(token),
    ///   ()      => console.log("done"),
    ///   (err)   => console.error(err),
    /// );
    /// ```
    #[napi]
    pub fn chat_stream(
        &self,
        request: JsLlmRequest,
        on_token: Function<'_, String, ()>,
        on_complete: Function<'_, (), ()>,
        on_error: Function<'_, String, ()>,
    ) -> napi::Result<()> {
        let tsfn_token = make_tsfn_string(&on_token)?;
        let tsfn_complete = make_tsfn_unit(&on_complete)?;
        let tsfn_error = make_tsfn_string(&on_error)?;

        let rust_req: ambi::types::LLMRequest = match request.try_into() {
            Ok(r) => r,
            Err(e) => {
                tsfn_error.call(e.clone(), ThreadsafeFunctionCallMode::Blocking);
                return Err(napi::Error::from_reason(e));
            }
        };

        let engine = Arc::clone(&self.inner);

        tokio::spawn(async move {
            let (tx, mut rx) =
                tokio::sync::mpsc::channel::<Result<String, ambi::error::AmbiError>>(64);

            tokio::spawn(async move {
                engine.chat_stream(rust_req, tx).await;
            });

            while let Some(chunk) = rx.recv().await {
                match chunk {
                    Ok(text) => {
                        tsfn_token.call(text, ThreadsafeFunctionCallMode::NonBlocking);
                    }
                    Err(e) => {
                        tsfn_error.call(e.to_string(), ThreadsafeFunctionCallMode::NonBlocking);
                        return;
                    }
                }
            }

            tsfn_complete.call((), ThreadsafeFunctionCallMode::NonBlocking);
        });

        Ok(())
    }

    // ── Context & Metadata ──

    /// Drop the engine's internal context (KV cache, accumulated state).
    /// Mostly relevant for local inference engines; a no-op for API backends.
    #[napi]
    pub fn reset_context(&self) {
        self.inner.reset_context();
    }

    /// Whether the underlying backend can process images.
    #[napi]
    pub fn supports_multimodal(&self) -> bool {
        self.inner.supports_multimodal()
    }

    /// Score a sentence's information entropy. Only makes sense with
    /// local engines that expose internal log-probabilities.
    #[napi]
    pub async fn evaluate_sentence_entropy(&self, sentence: String) -> napi::Result<f64> {
        self.inner
            .evaluate_sentence_entropy(&sentence)
            .await
            .map(|v| v as f64)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Quick token estimate without hitting the model — uses the built-in
    /// BPE tokenizer under the hood.
    #[napi]
    pub fn count_tokens(&self, text: String) -> napi::Result<u32> {
        self.inner
            .count_tokens(&text)
            .map(|c| c as u32)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }
}
