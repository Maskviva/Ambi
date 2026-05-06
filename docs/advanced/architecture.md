# Architecture Overview

This page explains how Ambi works under the hood. You don't need to know all of this to use the framework, but it helps if you want to extend it.

## Agent and AgentState are separate

This is the most important design decision. `Agent` is a read-only blueprint. `AgentState` is mutable conversation memory.

```
Agent (read-only, all fields Arc-wrapped → zero-cost clone)
├── LLMEngine              → the model backend
├── AgentConfig            → system prompt, template, eviction strategy
├── tools_def / tool_map   → registered tools and their definitions
├── tool_parser            → how tool calls are parsed from LLM output
├── cached_tool_prompt     → pre-rendered tool instruction string
├── formatter_factory      → how stream output is cleaned up
└── on_evict_handler       → callback for evicted messages (receives &AgentState)

AgentState (mutable, RwLock)
├── session_id             → unique conversation identifier (KV cache slotting, tracing)
├── dynamic_context        → volatile session data (RAG results, env vars)
├── chat_history           → pure FIFO queue of User / Assistant / Tool events
└── extensions             → anymap2 for custom state
```

**What changed in v0.3.3:**

- `session_id` was added to `AgentState` for distributed tracing and KV cache slotting in highly
  concurrent environments.
- `dynamic_context` was added to `AgentState` to handle volatile data (RAG, timestamps) separately
  from the static `system_prompt` in `AgentConfig`.
- `Message::System` was purged from `ChatHistory`. The history is now a pure FIFO queue of `User`,
  `Assistant`, and `Tool` events, streamlining the context eviction algorithm to O(1) truncation.
- Agent fields (`config`, `cached_tool_prompt`) are wrapped in `Arc` for truly zero-cost cloning
  across Tokio tasks.

This separation means:

- **One Agent, many conversations** – clone is just an Arc refcount bump
- **The Agent build happens once** – including blocking engine loading
- **State is fully serializable** – you can persist/restore conversations
- **Maximized KV Cache hit rates** – system prompt (static) is never evicted from the head

## The ReAct loop

When you call `runner.chat()` or `runner.chat_stream()`, this happens:

```
User Input
    │
    ▼
┌──────────────────────────────────────────┐
│ 1. Push user message to ChatHistory      │
│ 2. Build LLMRequest                      │
│    ├─ system_prompt + dynamic_context    │
│    ├─ cached_tool_prompt                 │
│    ├─ filtered history (User/Asst/Tool)  │
│    ├─ formatted_prompt string            │
│    └─ extracted images                   │
└──────────┬───────────────────────────────┘
           │
           ▼
┌──────────────────────────────────────────┐
│ 3. LLMEngine.chat() / chat_stream()      │
│    └─ Returns raw text                   │
└──────────┬───────────────────────────────┘
           │
           ▼
┌──────────────────────────────────────────┐
│ 4. ToolCallParser.parse(output)          │
│    └─ Extracts tool calls from text      │
└──────────┬───────────────────────────────┘
           │
    ┌──────┴──────────┐
    ▼                  ▼
No tools?         Tools found?
    │                  │
    ▼                  ▼
Return text   ┌────────────────────────────┐
              │ 5. Parallel execution      │
              │    .buffered(max_concurrency)│
              │    timeout per tool        │
              │    ghost cancellation      │
              └──────────┬─────────────────┘
                         │
                         ▼
              ┌────────────────────────────┐
              │ 6. Push tool results       │
              │    back to ChatHistory     │
              │    as Tool messages        │
              └──────────┬─────────────────┘
                         │
                         ▼
              ┌────────────────────────────┐
              │ 7. Eviction check          │
              │    Pure FIFO (no System)   │
              │    on_evict(state, msgs)   │
              └──────────┬─────────────────┘
                         │
                         ▼
              ┌────────────────────────────┐
              │ 8. Loop back to step 3     │
              │    (max_iterations)        │
              └────────────────────────────┘
```

Steps 3–8 repeat until either: no tool calls are produced, or `max_iterations` is reached.

### ChatRunner concurrency control

`ChatRunner` now holds `maximum_concurrency` (default 5 via `ChatRunner::default()`), allowing
flexible rate-limiting for parallel ghost-tool executions. You can create a custom runner:

```rust
use ambi::ChatRunner;

// Default: max 5 concurrent tool executions
let runner = ChatRunner::default();

// Custom limit
let runner = ChatRunner::new(3);
```

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
