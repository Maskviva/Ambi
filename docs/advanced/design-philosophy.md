# Design Philosophy

The evolution of Ambi's architecture is guided by four core principles. Understanding these helps you make better decisions when extending the framework.

---

## 1. Freedom Through Replaceability — Everything is a Trait

Ambi rejects "convention over configuration" as an implicit lock-in. **LLM backends, tool parsers, stream formatters, and even the entire execution pipeline are all exposed as replaceable traits.**

| Component | Trait | How to swap |
|-----------|-------|-------------|
| LLM backend | `LLMEngineTrait` | `LLMEngineConfig::Custom(...)` |
| Tool call parser | `ToolCallParser` | `agent.with_tool_parser(MyParser)` |
| Stream formatter | `StreamFormatter` | `agent.with_stream_formatter(|| Box::new(MyFormatter))` |
| Execution pipeline | `Pipeline` | `Pipeline.custom(handler)` or implement `Pipeline` |
| Tokenizer | `TokenizerTrait` | `engine.with_custom_tokenizer(MyTokenizer)` |

You can switch between local `llama.cpp` and cloud OpenAI with a single-line config change, or implement `LLMEngineTrait` to integrate any private model. This **trait-first** approach makes the framework a set of composable wheels, not a sealed-hood vehicle.

```rust
// One line to swap engines:
let config = LLMEngineConfig::OpenAI(openai_cfg);
// vs
let config = LLMEngineConfig::Llama(llama_cfg);
```

## 2. Extreme Developer Experience — Five Lines to Production

From day one, Ambi has aimed for **production-grade agents with minimal code**. Five lines of core code launch a cloud-reasoning chat agent:

```rust
let agent = Agent::make(config).await?
    .preamble("You are a helpful assistant.");
let state = AgentState::new_shared("session-1");
let runner = ChatRunner::default();
let reply = runner.chat(&agent, &state, "Hello!").await?;
```

The `#[tool]` procedural macro further reduces tool definition to a regular async Rust function, auto-generating JSON Schema, timeout, and retry logic:

```rust
#[tool(name = "get_weather", description = "Get weather for a city")]
async fn get_weather(city: String) -> Result<WeatherOutput, ToolErr> {
    // Your logic — no trait impl, no manual schema
}
```

This isn't magic — it's carefully designed zero-cost abstractions and smart defaults that push boilerplate into the framework's foundation.

## 3. Unbreakable Robustness — Production is Not a Playground

Ambi has a near-obsessive focus on failure modes hidden in the long tail:

### OOM Protection
The streaming buffer has a hard cap (`max_buffer_size`, default 8192). When the buffer exceeds it, it self-clears and logs an error, preventing runaway generation from blowing up memory.

### Smart Context Eviction
Evolved from early string-length estimation to an O(1) FIFO algorithm backed by a precise token accumulator:

```
each message → (Arc<Message>, exact_token_count)
total_tokens: cached sum → O(1) lookup
eviction: pure FIFO, no System message eviction
```

`User` messages are preserved as safe cut points — the history only stores `User`, `Assistant`, and `Tool` messages (System is fully decoupled), maintaining conversation coherence while preventing token overflow.

### Tool Execution Safety Net
- **Non-idempotent tools never retry** (no duplicate payments)
- **Idempotent tools** retry up to a configurable count
- **Client disconnect immediately aborts ghost tool execution** — built into the framework via `tokio::select!`
- **Cancellation safety** is guaranteed at the framework level

```rust
// Core safety pattern from tool_handler.rs:
tokio::select! {
    res = run_future => { /* tool completed */ }
    _ = async { tx_clone.closed().await } => {
        // Client disconnected. Aborting ghost tool execution.
    }
}
```

### Inference State Integrity
The local engine (`llama.cpp`) provides a `snapshot/restore` mechanism. After any decoding failure or stream interruption, both the KV Cache and session state are restored to a consistent point, eliminating silent state corruption.

```rust
// From session.rs:
pub fn snapshot(&self) -> (Vec<LlamaToken>, Vec<u8>, i32);
pub fn restore(&mut self, snapshot: ...);
```

## 4. Write Once, Run Anywhere — Unified Mental Model for Native and WASM

Ambi has a built-in cross-platform async runtime abstraction (`runtime.rs`) that seamlessly maps Tokio-native `spawn_blocking` to `wasm-bindgen-futures` polyfills:

| Function | Native (Tokio) | WASM |
|----------|---------------|------|
| `spawn` | `tokio::spawn` | `wasm_bindgen_futures::spawn_local` |
| `spawn_blocking` | `tokio::task::spawn_blocking` | Direct execution (single-threaded) |
| `sleep` | `tokio::time::sleep` | `gloo_timers::future::sleep` |
| `timeout` | `tokio::time::timeout` | Future race with timer |
| `Send + Sync` | Enforced | Relaxed (empty marker) |

Agent code written on a server compiles to the browser nearly unchanged. Hardware-dependent features (`llama.cpp`) are compile-time gated:

```rust
#[cfg(all(target_arch = "wasm32", feature = "llama-cpp"))]
compile_error!("The 'llama-cpp' feature is not supported on wasm32");
```

The same API, the same traits — seamlessly portable between cloud, edge, and device. This is the "universal AI framework" gene that Ambi has carried since 0.3.0.

---

## Summary

| Principle | Core Idea | Code Evidence |
|-----------|-----------|---------------|
| Freedom | Everything is a trait | `LLMEngineTrait`, `Pipeline`, `ToolCallParser`, `StreamFormatter` |
| DX | Five lines to production | `#[tool]` macro, fluent `Agent::make().preamble().template()` |
| Robustness | Production-grade safety net | FIFO eviction, ghost cancellation, snapshot/restore, OOM guard |
| Portability | Native ↔ WASM | `runtime.rs` polyfills, compile-time feature gating |
