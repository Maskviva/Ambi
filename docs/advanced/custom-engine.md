# Custom engine: replace inference without replacing the runtime

Ambi separates *how a model is called* from *how a request is executed*. Implement `LLMEngineTrait` for proprietary inference, deterministic tests, or local servers that do not expose an OpenAI-compatible API.

## Contract (0.4.1)

```rust
use ambi::{impl_as_any, LLMEngineConfig};
use ambi::llm::LLMEngineTrait;
use ambi::types::LLMRequest;
use ambi::error::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

struct EchoEngine;

#[async_trait]
impl LLMEngineTrait for EchoEngine {
    impl_as_any!();

    async fn chat(&self, _req: LLMRequest) -> Result<String> {
        Ok("Echo from custom engine".into())
    }

    async fn chat_stream(&self, _req: LLMRequest, tx: Sender<Result<String>>) {
        let _ = tx.send(Ok("Echo from custom engine".into())).await;
    }

    fn reset_context(&self) { /* stateless backend */ }
}

let cfg = LLMEngineConfig::Custom(Box::new(EchoEngine));
```

This snippet assumes `ambi`, `async-trait` and Tokio dependencies in a native project. Pass the config to `Agent::make(cfg).await?`. `supports_multimodal()` defaults to `false`; override it only when your backend truly supports image input. `as_any()` is required and the `impl_as_any!()` macro provides its implementation.

The primary output contract is **text (`String`)**, not a universal structured event stream. Tool-call text must agree with your `ToolCallParser`, which is separately replaceable.

## Access the engine from a custom pipeline

`Agent` does not expose mutable internal engine fields. Use the public getter:

```rust
let engine = agent.llm_engine(); // Arc<LLMEngine>
```

For a custom tokenizer, build a wrapper with `LLMEngine::load(...)` and `.with_custom_tokenizer(...)`, then use `Agent::from_engine(Arc::new(engine))`. Tokenizer counts are used for **approximate eviction accounting**; they are not necessarily exact model token counts.

## Runtime operational notes

- The default OpenAI-compatible backend has a concurrency ceiling and request timeout.
- The local llama.cpp backend has a worker queue and session-scoped cache ownership checks.
- `reset_context()` acts on the engine; on shared local engines this is not a per-session operation.
- A streaming error should be sent as `Err` on the channel. Be deliberate about terminal error handling and cancellation.

`Agent::with_custom_engine` and `LLMEngine::from_custom` are deprecated; prefer `LLMEngineConfig::Custom`.

See [Custom Pipeline](/advanced/custom-pipeline) · [Architecture](/advanced/architecture).
