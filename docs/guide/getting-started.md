# Getting Started

This page walks you through a working setup in about 5 minutes. You'll need:

- Rust 1.75+
- An API key (for cloud backends) or a GGUF model file (for local inference)

## 1. Add the dependency

```toml
[dependencies]
ambi = "0.3"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Ambi defaults to the `openai-api` feature. If you only ever use cloud backends, this keeps compilation fast.

For local inference via llama.cpp:

```toml
ambi = { version = "0.3", default-features = false, features = ["llama-cpp"] }
```

GPU acceleration is available as sub-features: `cuda`, `vulkan`, `metal`, `rocm` – pick exactly one.

```toml
ambi = { version = "0.3", features = ["llama-cpp", "cuda"] }
```

## 2. Minimal agent – 10 lines

```rust
use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LLMEngineConfig::OpenAI(ambi::OpenAIEngineConfig {
        api_key: std::env::var("OPENAI_API_KEY")?,
        base_url: "https://api.openai.com/v1".into(),
        model_name: "gpt-4o".into(),
        temp: 0.7,
        top_p: 0.95,
    });

    let agent = Agent::make(config).await?
        .preamble("You are a helpful assistant.")
        .template(ambi::ChatTemplateType::Chatml);

    let state = std::sync::Arc::new(tokio::sync::RwLock::new(AgentState::new()));
    let runner = ChatRunner;

    let reply = runner.chat(&agent, &state, "Hello!").await?;
    println!("{}", reply);

    Ok(())
}
```

That's it. `Agent::make` loads the engine (spawned on a blocking thread for llama.cpp), then the builder lets you chain configuration. `ChatRunner` executes the full ReAct loop.

## 3. Pick your engine

Switching between cloud and local is a one-line change – swap the config enum variant:

```rust
// Cloud
let config = LLMEngineConfig::OpenAI(openai_cfg);

// Local (requires "llama-cpp" feature)
let config = LLMEngineConfig::Llama(llama_cfg);
```

Everything else – tools, templates, streaming, formatters – stays the same.

## 4. Runtime requirement

Ambi needs Tokio with the `rt-multi-thread` feature. Single-threaded runtimes (`current_thread`) will not work because `Agent::make` calls `spawn_blocking` internally (needed for llama.cpp model loading).

## What's next

- [Basic Agent](/guide/basic-agent) – system prompts, chat templates, multi-turn conversations
- [Tools](/guide/tools) – giving your agent the ability to call Rust functions
- [Configuration](/guide/configuration) – eviction strategy, iteration limits, and more
