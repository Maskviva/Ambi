# Architecture Overview

This page explains how Ambi works under the hood. You don't need to know all of this to use the framework, but it helps if you want to extend it.

## Agent and AgentState are separate

This is the most important design decision. `Agent` is a read-only blueprint. `AgentState` is mutable conversation memory.

```
Agent (read-only, Arc-shared)
├── LLMEngine              → the model backend
├── AgentConfig            → system prompt, template, eviction strategy
├── tools_def / tool_map   → registered tools and their definitions
├── tool_parser            → how tool calls are parsed from LLM output
├── formatter_factory      → how stream output is cleaned up
└── on_evict_handler       → callback for evicted messages

AgentState (mutable, RwLock)
└── ChatHistory            → list of (Message, token_count)
```

This separation means:

- **One Agent, many conversations** – clone is just an Arc refcount bump
- **The Agent build happens once** – including blocking engine loading
- **State is fully serializable** – you can persist/restore conversations

## The ReAct loop

When you call `runner.chat()` or `runner.chat_stream()`, this happens:

```
User Input
    │
    ▼
┌──────────────────────────────────────┐
│ 1. Push user message to ChatHistory  │
│ 2. Build LLMRequest                  │
│    ├─ system_prompt + tool prompt    │
│    ├─ filtered history               │
│    ├─ formatted_prompt string        │
│    └─ extracted images               │
└──────────┬───────────────────────────┘
           │
           ▼
┌──────────────────────────────────────┐
│ 3. LLMEngine.chat() / chat_stream()  │
│    └─ Returns raw text               │
└──────────┬───────────────────────────┘
           │
           ▼
┌──────────────────────────────────────┐
│ 4. ToolCallParser.parse(output)      │
│    └─ Extracts tool calls from text  │
└──────────┬───────────────────────────┘
           │
    ┌──────┴──────┐
    ▼              ▼
No tools?     Tools found?
    │              │
    ▼              ▼
Return text   ┌────────────────────────┐
              │ 5. Parallel execution  │
              │    .buffered(5)        │
              │    timeout per tool    │
              │    ghost cancellation  │
              └──────────┬─────────────┘
                         │
                         ▼
              ┌────────────────────────┐
              │ 6. Push tool results   │
              │    back to ChatHistory │
              │    as Tool messages    │
              └──────────┬─────────────┘
                         │
                         ▼
              ┌────────────────────────┐
              │ 7. Eviction check      │
              │    FIFO if over budget │
              │    on_evict callback   │
              └──────────┬─────────────┘
                         │
                         ▼
              ┌────────────────────────┐
              │ 8. Loop back to step 3 │
              │    (max_iterations)    │
              └────────────────────────┘
```

Steps 3–8 repeat until either: no tool calls are produced, or `max_iterations` is reached.

## Template rendering

`ChatTemplate` defines how messages are serialized into the raw prompt string. Each variant stores prefix/suffix tags for system, user, assistant, and tool roles.

```
Example: ChatML format
──────────────────────
<|im_start|>system
You are helpful.
<|im_end|>
<|im_start|>user
Hello
<|im_end|>
<|im_start|>assistant
Hi there
<|im_end|>
<|im_start|>assistant   ← generation starts here
```

The engine receives the rendered prompt string. OpenAI engines additionally receive the structured `LLMRequest` with separated system/history/tools fields.

## Pipeline trait

`Pipeline` is the trait that defines the execution contract. `ChatRunner` is the built-in implementation, but you can write your own:

```rust
pub trait Pipeline {
    fn execute(&self, agent, state, input) -> impl Future<Output = Result<String>>;
    fn execute_stream(&self, agent, state, input)
        -> impl Future<Output = Result<Pin<Box<ReceiverStream<Result<String>>>>>>;
}
```

The pipeline has two modes:
- **Sync** – blocks until the full response is ready (internally runs the same ReAct loop)
- **Stream** – returns a `ReceiverStream` that the caller can iterate

## Extension points (all trait-based)

| What you can replace | Trait | Default |
|----------------------|-------|---------|
| LLM backend | `LLMEngineTrait` | OpenAI / llama.cpp |
| Tool implementation | `Tool` | None (you provide) |
| Tool call parsing | `ToolCallParser` | Tag-based `[TOOL_CALL]` |
| Stream formatting | `StreamFormatter` | Passthrough |
| Execution pipeline | `Pipeline` | `ChatRunner` |
| Tokenizer | `TokenizerTrait` | `cl100k_base` (tiktoken) |

## Cross-platform runtime

The `runtime` module abstracts platform differences:

| Function | Native (tokio) | WASM |
|----------|----------------|------|
| `spawn` | `tokio::spawn` | `wasm_bindgen_futures::spawn_local` |
| `spawn_blocking` | `tokio::task::spawn_blocking` | Direct execution (single-threaded) |
| `sleep` | `tokio::time::sleep` | `gloo_timers::future::sleep` |
| `timeout` | `tokio::time::timeout` | Future race with timer |
| `SendSync` | `Send + Sync` | Empty trait (no-op) |

For WASM, the `llama-cpp` feature is compile-time blocked:
```rust
#[cfg(all(target_arch = "wasm32", feature = "llama-cpp"))]
compile_error!("llama-cpp not supported on wasm32");
```
