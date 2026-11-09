# Architecture: separate capabilities, state and execution

Ambi's core is intentionally small. It is not a workflow graph or a multi-agent registry. It provides a reusable **Agent**, a caller-owned **AgentState**, and a replaceable **Pipeline** that defines execution.

Before diving into the code, read [Design philosophy](/advanced/design-philosophy); [Leaf](/guide/leaf) describes an optional instance of these ideas.

## The three essential boundaries

```text
                    +-----------------------------+
                    | Agent                       |
                    | Arc<LLMEngine>              |
                    | config / tools / parser     |
                    | stream formatter factory    |
                    +--------------+--------------+
                                   |
         +-------------------------+--------------------------+
         |                                                    |
 +-------v----------------+                         +---------v----------+
 | ChatRunner : Pipeline  |                         | Your Pipeline       |
 | ReAct + tool execution |                         | your execution rule |
 +-----------+------------+                         +---------+----------+
             |                                                |
             +--------------------+---------------------------+
                                  |
                       +----------v----------+
                       | AgentState          |
                       | session_id          |
                       | chat_history        |
                       | dynamic_context     |
                       | extensions          |
                       +---------------------+
```

`Agent` is cheap to clone because its internals are shared with `Arc`. `AgentState` is serialized with `serde` except its type-erased extension map. `AgentState::new_shared()` returns `Arc<RwLock<AgentState>>`; you should normally allocate **one independent state per conversation**.

The same `Agent` can serve many states. **Do not interpret the `RwLock` as a run-level transaction**: separate operations lock/unlock the state independently. Concurrent requests using the exact same state require application-level ordering if you need a strictly sequential conversation.

## A `ChatRunner` request

1. Validate input (e.g. image support and non-zero tool concurrency).
2. Resolve tools. With an opted-in Leaf, augment the parent tool snapshot for that run.
3. Append the user message to `AgentState.chat_history`.
4. Build `LLMRequest` from the system prompt, dynamic context, tool definitions, filtered history, template and images. It includes `session_id`.
5. Invoke the engine; parse tool requests from model output.
6. Execute tool calls with a per-loop concurrency bound and per-tool timeout/retry policy.
7. Feed tool results back into history, continue until a final response or iteration limit.
8. Account for history tokens and evict old messages according to the configured budget.

`chat()` returns `Result<String>`; `chat_stream()` returns a stream of `Result<String>` chunks. They share the same internal ReAct loop but expose different output delivery semantics.

## Engine boundary

`LLMEngineTrait` defines `chat(LLMRequest) -> Result<String>`, `chat_stream(LLMRequest, Sender<Result<String>>)`, `reset_context()` and `supports_multimodal()` (plus downcasting). It is intentionally small and text-oriented; it does **not** expose a universal structured model event protocol.

- **OpenAI-compatible API:** HTTP backend with a concurrency semaphore and request timeout; image capability is configured explicitly or inferred for recognized model prefixes.
- **llama.cpp:** native-only worker thread with bounded commands, cancellation checks and a shared KV cache. `session_id` helps prevent treating a different conversation as the same cached prefix. Concurrent sessions may incur re-prefill.
- **Custom:** provide `LLMEngineConfig::Custom(Box<dyn LLMEngineTrait>)`.

`Agent::llm_engine()` exposes an `Arc<LLMEngine>` for custom pipeline authors without exposing the agent's private fields.

## Leaf shares mechanisms, not identity

The optional Leaf is built on `ChatRunner` and the `Tool` mechanism. The parent sees the original tools plus `spawn_leaf`. The Leaf has the original tool snapshot **without** `spawn_leaf`, and executes `run_loop` with a fresh `AgentState` and `final_answer_only = true`.

It is not a persistent child Agent. It does not inherit parent chat history; it **does** share the inference engine and tools. See [Leaf task delegation](/guide/leaf) for API and safety constraints.

## Runtime guarantees and non-guarantees

| Topic | Current behavior | Caveat |
| --- | --- | --- |
| State | Session history and dynamic context are separate from Agent | No automatic full-run lock on a shared state |
| Eviction | FIFO messages with cached approximate token totals | Not atomic tool-call-group eviction or an exact provider token guarantee |
| Tool retries | Timeout retries are allowed only for tools marked idempotent | Cancellation cannot undo external side effects |
| Leaf | Temporary state and final-answer-only result | Shares engine and original tool permissions |
| Pipelines | Implementable `Pipeline` trait | No built-in arbitrary DAG composition |
| Streaming | Response channel with cancellation-sensitive processing | Remote requests and external operations may have their own cancellation semantics |

The runtime is a substrate for application-defined behavior. If your application requires audit-grade effect tracking, durable task queues or access controls, build them explicitly above these mechanisms.

## Source map

| Concern | Code |
| --- | --- |
| Agent / State | `src/agent/core.rs`, `core/builder.rs` |
| Pipeline / runner | `src/agent/pipeline.rs`, `pipeline/chat_runner.rs` |
| ReAct loop | `src/agent/pipeline/chat_runner/multimodal_handler.rs` |
| Leaf | `src/agent/pipeline/chat_runner/leaf.rs` |
| Tools | `src/types/tool_def.rs`, `src/agent/tool/manager.rs` |
| Engines | `src/llm/engine.rs`, `src/llm/providers/` |
| History | `src/agent/core/history.rs` |

Next: [Custom Pipeline](/advanced/custom-pipeline) · [Custom Engine](/advanced/custom-engine) · [Context Eviction](/advanced/context-eviction).
