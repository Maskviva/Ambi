# Getting started (Rust, 0.4.1)

This is the shortest route to a working text agent. Ambi is an **Agent Runtime**, not a mandatory multi-agent framework: you choose an engine, build reusable capabilities into an `Agent`, pass separate `AgentState`, and select a `Pipeline`.

## 1. Add dependencies

```toml
[dependencies]
ambi = "0.4.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

The default feature is `openai-api`. If `0.4.1` is not yet published to your registry, use a `path` or Git dependency pointing to the matching source. Configure `OPENAI_API_KEY` for your endpoint.

## 2. Run a chat

Create `src/main.rs`:

```rust
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::var("OPENAI_API_KEY")?;
    let cfg = OpenAIEngineConfig::create(key, "gpt-4o-mini")
        .temp(0.7);

    let agent = Agent::make(LLMEngineConfig::OpenAI(cfg)).await?
        .preamble("You are a concise assistant.");
    let state = AgentState::new_shared("session-1");
    let runner = ChatRunner::default();

    let answer = runner.chat(&agent, &state, "Hello, Ambi!").await?;
    println!("{answer}");
    Ok(())
}
```

Run with `cargo run`. `Agent::make` initializes the engine; `ChatRunner::default()` uses a ReAct-style chat/tool loop and a default per-loop tool concurrency of five.

## 3. Choose a backend

**OpenAI-compatible endpoint:**

```rust
let cfg = OpenAIEngineConfig::create(api_key, "your-model")
    .base_url("https://your-endpoint.example/v1".into())
    .temp(0.2);
```

**Local GGUF via llama.cpp** (native targets only):

```toml
ambi = { version = "0.4.1", default-features = false, features = ["llama-cpp"] }
```

```rust
use ambi::llm::providers::llama_cpp::config::LlamaEngineConfig;
let cfg = LlamaEngineConfig::builder("./models/model.gguf")
    .n_ctx(4096)
    .n_tokens(512)
    .build();
let engine_cfg = LLMEngineConfig::Llama(cfg);
```

You can optionally enable one GPU backend (`cuda`, `vulkan`, `metal`, or `rocm`). Do not select incompatible accelerators together. Browser WebAssembly uses the OpenAI-compatible backend, not llama.cpp.

## 4. Continue from here

- [Design philosophy](/advanced/design-philosophy) — why Agent, Pipeline and Leaf are distinct
- [Basic Agent](/guide/basic-agent) — separate conversations and dynamic context
- [Tools](/guide/tools) — expose typed Rust operations
- [Leaf](/guide/leaf) — optional, disposable task delegation (`sub-agent` feature)
- [Custom Pipeline](/advanced/custom-pipeline) — decide how the agent executes

Examples in the repository: [`examples/chat_cloud.rs`](https://github.com/Maskviva/Ambi/blob/main/examples/chat_cloud.rs), [`examples/chat_local.rs`](https://github.com/Maskviva/Ambi/blob/main/examples/chat_local.rs).
