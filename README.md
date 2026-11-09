# Ambi

<p align="center">
  <img src="docs/public/logo.svg" alt="Ambi logo" width="96" />
</p>

<p align="center">
  <strong>A small, composable AI Agent Runtime for Rust.</strong><br />
  Build the agent you need. Keep control of how it runs.
</p>

<p align="center">
  <a href="https://crates.io/crates/ambi"><img alt="crates.io" src="https://img.shields.io/crates/v/ambi" /></a>
  <a href="https://docs.rs/ambi"><img alt="docs.rs" src="https://img.shields.io/docsrs/ambi" /></a>
  <a href="LICENSE"><img alt="Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue" /></a>
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.zh.md">简体中文</a> ·
  <a href="https://maskviva.github.io/Ambi/">Documentation</a> ·
  <a href="CHANGELOG.md">Changelog</a> ·
  <a href="examples/">Examples</a>
</p>

---

**Ambi** provides the runtime building blocks for LLM-powered agents without prescribing how every agent must reason,
plan, or cooperate. It is written in Rust, built around replaceable traits, and designed to remain small while
applications grow around it.

Use a ready-made `ChatRunner` for ReAct-style tool use, or implement your own `Pipeline`. Choose an OpenAI-compatible
API, run a local GGUF model through `llama.cpp`, or provide your own engine. Compose the capabilities you need; leave
out the ones you don't.

> **Design premise:** Composition over inheritance. Ambi supplies execution mechanisms, not a mandatory cognitive
> architecture. Its built-in use cases are a starting point—not a ceiling on what you can build with its traits.

## Why Ambi?

- **Small core, open extension points.** Replace the model engine (`LLMEngineTrait`), execution strategy (`Pipeline`),
  tool parser, stream formatter, or tokenizer without adopting a larger agent hierarchy.
- **Agent and state are separate.** `Agent` holds shared capabilities and configuration; `AgentState` carries
  per-session history and dynamic context. A reusable agent blueprint need not own every conversation.
- **Tools are first-class.** Async Rust tools support schema-defined inputs, concurrent execution, timeouts, and retry
  policy that distinguishes idempotent operations.
- **Cloud and local backends.** OpenAI-compatible Chat Completions APIs, local `llama.cpp` inference, and custom engines
  share the same high-level agent interface.
- **Control over context.** Token-accounted conversation history, FIFO eviction, dynamic context, and optional eviction
  callbacks are available without forcing a memory database.
- **Cross-platform where supported.** Native Rust and browser WebAssembly share the core API; the `llama-cpp` backend is
  native-only.
- **Leaf task delegation (opt-in).** A pipeline can create short-lived, isolated task executions instead of
  instantiating a second full agent hierarchy.

## The mental model

```text
                      Agent
         (engine, tools, prompt, configuration)
                        │
             ┌──────────┴──────────┐
             │                     │
         ChatRunner           Your Pipeline
          (ReAct)             (trait-based)
             │
       optional spawn_leaf
             │
      ┌──────┴──────┐
      │             │
    Leaf A        Leaf B          → final answers only
 (fresh state)  (fresh state)

  AgentState: separate conversation state passed to a pipeline
```

This tree is a **design metaphor**, not a required inheritance hierarchy. An `Agent` is the reusable tree, pipelines are
its branches, and a *Leaf* is a disposable execution grown for one task. Other developers are free to implement
different branches and capabilities using Ambi's traits.

| Building block      | Responsibility                                                 |
|---------------------|----------------------------------------------------------------|
| `Agent`             | Share an engine, system prompt, tools, and configuration       |
| `AgentState`        | Hold session-specific conversation history and dynamic context |
| `Pipeline`          | Define how an agent executes a request                         |
| `ChatRunner`        | Provide the built-in ReAct-style chat / tool loop              |
| `LLMEngineTrait`    | Abstract model inference behind a replaceable engine           |
| `Tool`              | Expose typed external operations to the model                  |
| `Leaf` *(optional)* | Execute a temporary delegated task and return its final answer |

## Quick start

### 1. Install

For an OpenAI-compatible endpoint, the default feature is sufficient:

```toml
[dependencies]
ambi = "0.4.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

To depend on this repository before the version is available from your registry, use a local path or Git dependency
instead. The Rust examples below use the API present in the **0.4.1 source tree**.

### 2. Create an agent

Set `OPENAI_API_KEY` in your environment and put this in `src/main.rs`:

```rust
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let engine = OpenAIEngineConfig::create(api_key, "gpt-4o-mini")
        .temp(0.7);

    let agent = Agent::make(LLMEngineConfig::OpenAI(engine))
        .await?
        .preamble("You are a helpful assistant.");

    let state = AgentState::new_shared("conversation-1");
    let runner = ChatRunner::default();

    let answer = runner.chat(&agent, &state, "Hello, Ambi!").await?;
    println!("{answer}");
    Ok(())
}
```

`ChatRunner::default()` does **not** enable Leaf automatically, even when the `sub-agent` feature is compiled in.

Explore the repository examples: [`chat_cloud`](examples/chat_cloud.rs), [`chat_stream`](examples/chat_stream.rs), [
`chat_local`](examples/chat_local.rs), and [`custom_tool`](examples/custom_tool.rs).

## Leaf: delegation without another agent hierarchy

A Leaf exists for a **single delegated task**. The parent model sees it as a tool, `spawn_leaf`, but the implementation
is a temporary execution of the existing chat loop—not a new persistent `Agent`.

A Leaf:

1. Starts with a fresh `AgentState`, without the parent's conversation history.
2. Reuses the parent's engine and a snapshot of its original tools and chat configuration, with selected settings
   overridable by `LeafConfig`.
3. Cannot call `spawn_leaf` itself: its tool snapshot does not contain that tool, so recursion is structurally
   prevented.
4. Returns **only its final answer** to the parent; intermediate tool output and reasoning text are not copied into the
   parent's answer context.
5. Discards its temporary conversation state when the task finishes.

This is useful for bounded work such as summarizing a document, exploring a codebase, or comparing independent findings
while preserving the parent conversation's context budget.

**Enable the feature explicitly:**

```toml
ambi = { version = "0.4.1", features = ["sub-agent"] }
```

Then opt in **per runner**:

```rust
use ambi::{ChatRunner, LeafConfig};

// Simple opt-in: ChatRunner::default().with_leaf()
let runner = ChatRunner::default ().with_leaf_config(
LeafConfig::new()
.with_max_iterations(6)
.with_timeout_secs(120),
);

// Use it with an existing `agent` and `state`:
let answer = runner.chat(
& agent,
& state,
"Read the architecture notes and delegate a concise summary to a leaf.",
).await?;
```

The model decides whether to invoke the injected tool; `with_leaf()` does not unconditionally spawn a Leaf. Independent
Leaf calls in a tool-calling round can run concurrently, subject to backend and tool limits.

> **Scope and safety:** Leaf isolates **conversation history**, not model resources or external effects. It shares the
> parent's engine and **all parent tools**, including potentially side-effecting tools. Local `llama.cpp` requests share
> one inference worker/KV cache, so switching between parent and Leaf sessions may cost additional prefill. Leaf is not a
> sandbox or a persistent worker. The current built-in implementation does not provide a tool allow-list or alternate
> model selection.

See [`src/agent/pipeline/chat_runner/leaf.rs`](src/agent/pipeline/chat_runner/leaf.rs) for the current implementation.

## Give the agent tools

Implement `Tool` to expose an async Rust operation. A tool definition includes a JSON Schema, timeout, retry count, and
whether retrying is safe.

```rust
use ambi::types::{Tool, ToolDefinition, ToolErr};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct AddArgs {
    a: i64,
    b: i64
}

#[derive(Serialize)]
struct AddResult {
    sum: i64
}

struct AddNumbers;

#[async_trait]
impl Tool for AddNumbers {
    const NAME: &'static str = "add_numbers";
    type Args = AddArgs;
    type Output = AddResult;

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Add two integers".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": { "type": "integer" },
                    "b": { "type": "integer" }
                },
                "required": ["a", "b"]
            }),
            timeout_secs: Some(5),
            max_retries: Some(1),
            is_idempotent: true,
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr> {
        Ok(AddResult { sum: args.a + args.b })
    }
}

// While building your agent:
// let agent = Agent::make(config).await?.tool(AddNumbers)?;
```

Add `async-trait`, `serde` (with `derive`), and `serde_json` to your application's dependencies for this manual tool
example. For less boilerplate, enable the `macro` feature and see [
`examples/custom_tool_in_macro.rs`](examples/custom_tool_in_macro.rs). The macro can generate tool argument schema; the
manual `Tool` implementation above supplies its schema explicitly.

**Be careful with side effects.** Mark non-idempotent operations `is_idempotent: false`. Timeouts and cancellation do
not guarantee reversal of an operation already accepted by an external service.

## Streaming and context

The same `ChatRunner` exposes streaming output (add the `futures` dependency for `StreamExt`):

```rust
use futures::StreamExt;

let mut stream = runner.chat_stream( & agent, & state, "Explain this project").await?;
while let Some(chunk) = stream.next().await {
print!("{}", chunk?);
}
```

Session state is separate from the agent and may be passed across turns. Dynamic context is supplied independently of
stored chat history:

```rust
state.write().await.set_dynamic_context("Relevant project notes: ...");
// After this context is no longer relevant:
state.write().await.clear_dynamic_context();
```

Conversation-history eviction uses token accounting and removes older entries via FIFO when the configured budget is
exceeded. The system prompt is not part of that FIFO history. It is **not** a promise of exact provider token
accounting, semantically grouped eviction, or persistent long-term memory. Applications can register an `on_evict`
callback and supply their own persistence strategy.

## Backends, extensions, and platforms

### Cargo features

| Feature                              | Default? | Purpose                                                       |
|--------------------------------------|----------|---------------------------------------------------------------|
| `openai-api`                         | Yes      | OpenAI-compatible API backend                                 |
| `llama-cpp`                          | No       | Local GGUF inference through `llama.cpp`                      |
| `cuda` / `vulkan` / `metal` / `rocm` | No       | Choose **one** native GPU backend; each enables `llama-cpp`   |
| `mtmd`                               | No       | `llama.cpp` multimodal support                                |
| `dynamic-link`                       | No       | Dynamic linking option for the local backend                  |
| `macro`                              | No       | Procedural macros for tool/agent registration                 |
| `sub-agent`                          | No       | Compile in Leaf support; opt in at runtime with `with_leaf()` |

For local inference without an API dependency:

```toml
ambi = { version = "0.4.1", default-features = false, features = ["llama-cpp"] }
```

```rust
use ambi::llm::providers::llama_cpp::config::LlamaEngineConfig;
use ambi::LLMEngineConfig;

let local = LlamaEngineConfig::builder("./models/model.gguf")
.n_ctx(4096)
.n_tokens(512)
.max_tokens(512)
.build();
let config = LLMEngineConfig::Llama(local);
```

GPU backends require their corresponding toolchains/libraries. `llama-cpp` cannot be compiled for `wasm32`; browser
builds use `openai-api` or a custom engine. Browser API calls also depend on the endpoint's CORS policy; do not expose
privileged API keys in public clients.

### Optional crates and language bindings

Ambi keeps higher-level functionality outside the core crate:

| Package                                         | Role                                                                                       |
|-------------------------------------------------|--------------------------------------------------------------------------------------------|
| [`ambi-pipelines`](ambi-pipelines/)             | RAG, CoT self-consistency, Tree-of-Thoughts, Reflexion                                     |
| [`ambi-memory`](ambi-memory/)                   | KV, summary, and semantic-memory provider interfaces; in-memory KV/summary implementations |
| [`ambi-macros`](ambi-macros/)                   | Procedural macros for agent/tool definitions                                               |
| [`bindings/python`](bindings/python/)           | Python bindings (`ambi-python` package)                                                    |
| [`bindings/node`](bindings/node/)               | Node.js bindings (`@maskviva/ambi-node` package)                                           |
| [`examples/webAssembly`](examples/webAssembly/) | Browser/WASM example                                                                       |

Some extension APIs and language bindings have their own release versions and feature coverage; check their directories
for details. A semantic-memory provider interface does **not** imply an included vector database, and the in-memory
providers are not durable storage.

## Documentation and development

- [Getting started](docs/guide/getting-started.md) · [Custom tools](docs/guide/tools.md) · [Custom pipelines](docs/advanced/custom-pipeline.md)
- [Architecture](docs/advanced/architecture.md) · [Design philosophy](docs/advanced/design-philosophy.md) · [WASM](docs/platform/wasm.md)
- [Examples](examples/) · [Changelog](CHANGELOG.md) · [中文更新日志](CHANGELOG_zh.md)

```bash
cargo fmt --all -- --check
cargo clippy --no-default-features --features openai-api -- -D warnings
cargo test --no-default-features --features openai-api
cargo test --no-default-features --features "openai-api,sub-agent" --test leaf_subagent_test
```

The Leaf-specific test requires the `sub-agent` feature. Tests depending on external model services additionally require
a configured endpoint; a successful build or unit test is not a performance benchmark or a production guarantee.

## License

Licensed under [Apache License 2.0](LICENSE). Contributions are welcome; keep the core small, extension points clear,
and behavior explicit.
