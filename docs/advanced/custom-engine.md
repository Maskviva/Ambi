# Custom Engine

If the built-in OpenAI and llama.cpp engines don't cover your use case, you can bring your own backend.

## When to write a custom engine

- You want to use a proprietary model over a custom protocol
- You need a mock engine for tests (avoid real API calls)
- You're running a non-OpenAI-compatible local server

## Implementing `LLMEngineTrait`

```rust
use ambi::llm::LLMEngineTrait;
use ambi::types::LLMRequest;
use ambi::error::Result;
use tokio::sync::mpsc::Sender;
use async_trait::async_trait;

struct MockEngine {
    reply: String,
}

#[async_trait]
impl LLMEngineTrait for MockEngine {
    impl_as_any!(); // required for engine downcasting

    async fn chat(&self, _request: LLMRequest) -> Result<String> {
        Ok(self.reply.clone())
    }

    async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>) {
        // Send chunks
        let _ = tx.send(Ok("Hello, ".into())).await;
        let _ = tx.send(Ok("world!".into())).await;
    }

    fn reset_context(&self) {
        // no-op for a mock
    }
}
```

### Required methods

| Method | Purpose |
|--------|---------|
| `chat()` | Full response. Returns the complete output string. |
| `chat_stream()` | Streamed response. Send chunks via the mpsc `Sender`. |
| `reset_context()` | Clear any internal state/KV cache. Called by `ChatRunner::clear_history()`. |

### Optional methods

| Method | Default | Override when... |
|--------|---------|-----------------|
| `supports_multimodal()` | `false` | Your engine handles images |
| `as_any()` | Panics | Implement via `impl_as_any!()` for downcasting to concrete type |

## Using the custom engine

Pass your engine via the `LLMEngineConfig::Custom` variant:

```rust
use ambi::{Agent, LLMEngineConfig};

let agent = Agent::make(
    LLMEngineConfig::Custom(Box::new(MockEngine {
        reply: "Hello, I'm a mock.".into(),
    }))
).await?;
```

This is the **recommended** approach. The old `Agent::with_custom_engine()` is deprecated.

Note: `LLMEngineConfig::Custom` is synchronous – it doesn't need `spawn_blocking` because there's no model file to load. This also means it works in `current_thread` Tokio runtimes.

## Using a custom engine in tests

Mock engines are useful for deterministic testing of tool logic:

```rust
#[tokio::test]
async fn test_tool_calls() {
    let agent = Agent::make(LLMEngineConfig::Custom(Box::new(MockEngine {
        reply: "Tell me the weather[TOOL_CALL]{\"name\":\"get_weather\",\"args\":{\"city\":\"Tokyo\"}}[/TOOL_CALL]".into(),
    }))).await?;
    // ... test your tools
}
```

## Custom tokenizer

By default, Ambi uses `cl100k_base` (tiktoken). If your model uses a different tokenizer, swap it:

```rust
use ambi::llm::{LLMEngine, LLMEngineConfig};
use ambi::llm::tokenizer::TokenizerTrait;

struct MyTokenizer;

impl TokenizerTrait for MyTokenizer {
    fn count_tokens(&self, text: &str) -> Result<usize> {
        Ok(text.len()) // rough estimate
    }
}

// After creating the engine:
let engine = LLMEngine::load(LLMEngineConfig::Custom(Box::new(my_engine)))?;
let engine = engine.with_custom_tokenizer(MyTokenizer);
```

> **Note:** The old `LLMEngine::from_custom()` method is deprecated.
> Use `LLMEngine::load(LLMEngineConfig::Custom(backend))` instead.

This affects context eviction accuracy. An inaccurate tokenizer may evict too early or too late.
