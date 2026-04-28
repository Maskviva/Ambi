# Ambi

---
[](https://github.com/maskviva/ambi)
[](https://crates.io/crates/ambi)
[](https://docs.rs/ambi)

<p align="center">
  <a href="https://spdx.org/licenses/Apache-2.0.html"><img src="https://img.shields.io/badge/License-Apache%202.0-blue" alt="License: GPL v3"></a>
  <a href="https://github.com/maskviva/ambi"><img alt="github" src="https://img.shields.io/badge/github-maskviva/Ambi-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20"></a>
  <a href="https://crates.io/crates/ambi"><img alt="crates.io" src="https://img.shields.io/crates/v/ambi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20"></a>    
  <a href="https://docs.rs/ambi"><img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-ambi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20"></a><br/>
 [<a href="./README_zh.md">中文(简体)</a>] | [<a href="./README.md">English</a>] 
</p>

Ambi is a flexible, highly customizable AI Agent framework built entirely in Rust. It empowers you to create
production‑grade agents with minimal boilerplate, trait‑first design, and zero‑cost abstractions.

- **Dual‑engine architecture** – Seamlessly switch between local inference (via `llama.cpp` with GPU acceleration) and
  cloud APIs (OpenAI‑compatible endpoints) without changing your agent code.
- **Advanced tool system** – Parallel multi‑tool execution, per‑tool timeouts and retries, automatic JSON Schema
  generation from Rust structs.
- **Intelligent context management** – Safe eviction algorithm that preserves conversation logic, preventing token
  overflow while keeping your agent focused.
- **Rust native** – Memory safety, async/await everywhere, minimal dependencies, and fast compilation times.

<br>

## Resources

The best way to learn Ambi is to write an agent. The [`examples/`](https://github.com/maskviva/ambi/tree/main/examples)
directory contains complete, runnable examples covering basic chat, custom tools, local GPU inference, streaming, and
multi‑tool parallel execution.

<br>

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ambi = "0.2"
```

For cloud‑only usage (faster compilation, no `llama.cpp` dependency):

```toml
ambi = { version = "0.2", default-features = false, features = ["openai-api"] }
```

## Runtime Requirements

Ambi is built on the Tokio async runtime. Ensure your project uses Tokio with `rt-multi-thread` enabled. Without this,
`Agent::make` and all async methods will not function.

<br>

## Quick start

```rust
use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig, Message};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Pick an engine configuration
    let config = LLMEngineConfig::OpenAI(ambi::OpenAIEngineConfig {
        api_key: std::env::var("OPENAI_API_KEY")?,
        base_url: "https://api.openai.com/v1".into(),
        model_name: "gpt-4o".into(),
        temp: 0.7,
        top_p: 0.95,
    });

    // 2. Build an agent (5 lines of code!)
    let agent = Agent::make(config).await?
        .preamble("You are a helpful assistant.")
        .template(ambi::ChatTemplateType::Chatml);

    // 3. Create a shared state
    let state = Arc::new(Mutex::new(AgentState::new()));

    // 4. Run the chat pipeline
    let runner = ChatRunner;
    let response = runner.chat(&agent, &state, "Hello, world!").await?;
    println!("{}", response);

    Ok(())
}
```

<br>

## Using local inference

Enable the `llama-cpp` feature and optionally a GPU backend:

```toml
ambi = { version = "0.2", features = ["llama-cpp", "cuda"] }
```

Then swap the engine configuration:

```rust
let config = LLMEngineConfig::Llama(ambi::LlamaEngineConfig {
model_path: "./models/llama-3-8b.gguf".into(),
max_tokens:  4096,
buffer_size: 32,
use_gpu:     true,
n_gpu_layers: 100,
n_ctx:       8192,
n_tokens:    512,
n_seq_max:   1,
penalty_last_n:   64,
penalty_repeat:   1.1,
penalty_freq:     0.0,
penalty_present:  0.0,
temp:       0.7,
top_p:      0.9,
seed:       42,
min_keep:   1,
});
```

<br>

## Adding custom tools

Define a tool by implementing the `Tool` trait. Ambi automatically generates the JSON Schema for you.

```rust
use ambi::{Tool, ToolDefinition, ToolErr};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

#[derive(Deserialize)]
struct WeatherArgs {
    city: String,
}

#[derive(Serialize)]
struct WeatherResult {
    temperature: f64,
    condition: String,
}

struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    const NAME: &'static str = "get_weather";

    type Args = WeatherArgs;
    type Output = WeatherResult;

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "get_weather".into(),
            description: "Get current weather for a city".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "City name"
                    }
                },
                "required": ["city"]
            }),
            timeout_secs: Some(10),
            max_retries: Some(2),
            is_idempotent: true,
        }
    }

    async fn call(&self, args: WeatherArgs) -> Result<WeatherResult, ToolErr> {
        // Your implementation here
        Ok(WeatherResult {
            temperature: 22.5,
            condition: "Sunny".into(),
        })
    }
}
```

Attach the tool to your agent:

```rust
let agent = Agent::make(config).await?
.preamble("You are a weather assistant.")
.tool(WeatherTool) ?;
```

Now the agent can seamlessly invoke `get_weather` when the user asks about the weather. Ambi handles retries, timeouts,
and parallel execution automatically.

<br>

## Streaming responses

```rust
use futures::StreamExt;

let mut stream = runner.chat_stream( & agent, & state, "Tell me a story").await?;
while let Some(chunk) = stream.next().await {
match chunk {
Ok(text) => print ! ("{}", text),
Err(e) => eprintln !("Stream error: {}", e),
}
}
```

<br>

## Context eviction

Ambi’s context management automatically evicts old messages when the token budget is exceeded, but you can fine‑tune the
strategy:

```rust
let agent = Agent::make(config).await?
.with_eviction_strategy(
2,    // keep at least first 2 messages
6,    // keep at least most recent 6 messages
3000, // max safe tokens before eviction
);
```

You can also register a callback to process evicted messages (e.g., to persist them):

```rust
let agent = Agent::make(config).await?
.on_evict( | evicted| {
for msg in evicted {
// archive or log the message
}
});
```

<br>

## Custom tool‑call parser

By default Ambi uses `[TOOL_CALL] ... [/TOOL_CALL]` tags. You can bring your own parser:

```rust
use ambi::tool::{ToolCallParser, DefaultToolParser};

struct MyParser;

impl ToolCallParser for MyParser {
    fn format_instruction(&self, tools_json: &str) -> String {
        // instruct the model how to call tools
        format!("Use tools: {}", tools_json)
    }

    fn parse(&self, text: &str) -> Vec<(String, serde_json::Value)> {
        // extract tool calls from the model's output
        vec![]
    }

    fn create_stream_formatter(&self) -> Box<dyn ambi::tool::StreamFormatter> {
        Box::new(ambi::agent::core::formatter::PassThroughFormatter)
    }
}

let agent = Agent::make(config).await?
.with_tool_parser(MyParser);
```

<br>

## Error handling

Ambi uses `thiserror` to provide clear, actionable error types:

```rust
pub enum AmbiError {
    EngineError(String),
    AgentError(String),
    ToolError(String),
    ContextError(String),
    PipelineError(String),
    MaxIterationsReached(usize),
    Other(anyhow::Error),
}
```

All public APIs return `Result<T, AmbiError>`, making it easy to pattern‑match or propagate errors.

<br>

## Testing

Ambi comes with comprehensive unit and integration tests. We recommend using `cargo test` during development. When
testing agents, consider using a mock engine to avoid real API calls:

```rust
struct MockEngine;
#[async_trait]
impl LLMEngineTrait for MockEngine {
    async fn chat(&self, _: LLMRequest) -> Result<String> {
        Ok("Hello, I am a mock.".into())
    }
    // ...
}

let agent = Agent::with_custom_engine(Box::new(MockEngine)) ?;
```

<br>

## Feature flags

Ambi uses Cargo features to keep compile times low:

- **`openai-api`** *(enabled by default)* – OpenAI‑compatible cloud backend powered by `async-openai`.
- **`llama-cpp`** – Local inference via `llama.cpp` (supports `cuda`, `vulkan`, `metal`, `rocm` sub‑features).
- **`cuda`**, **`vulkan`**, **`metal`**, **`rocm`** – GPU acceleration for the local engine (choose exactly one).

<br>

#### License

<sup>
Licensed under the <a href="LICENSE-APACHE">Apache License, Version 2.0</a>.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be licensed as above, without any additional terms or conditions.
</sub>