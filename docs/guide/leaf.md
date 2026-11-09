# Leaf: disposable task delegation

Available in **Ambi 0.4.1**, behind the `sub-agent` Cargo feature. A Leaf is an opt-in capability of **`ChatRunner`**, not an independent `Agent` type and not a feature automatically enabled for other pipelines.

## Why a Leaf?

A parent agent may need to summarize a large file or inspect a module without adding every intermediate step to its own conversation. Calling another agent is one possible implementation; Ambi instead treats delegation as a **temporary execution grown from a pipeline**.

In the tree metaphor, Agent is the tree, Pipeline the branch, and Leaf a leaf: created for a task, used, then dropped. Other branches can have their own strategies. See [Design philosophy](/advanced/design-philosophy).

## Enable it in two steps

1. Compile the optional feature:

```toml
[dependencies]
ambi = { version = "0.4.1", features = ["sub-agent"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

2. Opt in on a specific runner:

```rust
use ambi::ChatRunner;

let runner = ChatRunner::default().with_leaf();
// The rest of the setup uses the same Agent and AgentState as ordinary chat.
```

`ChatRunner::default()` does **not** inject a Leaf tool merely because the Cargo feature was enabled. The model-facing tool is named `spawn_leaf` by default.

## Customize the delegation policy

```rust
use ambi::{ChatRunner, LeafConfig};

let cfg = LeafConfig::new()
    .with_tool_name("delegate_summary")
    .with_description("Summarize an independent document or code module; return only the conclusion.")
    .with_system_prompt("You are a concise research assistant. Report findings and uncertainties.")
    .with_max_iterations(6)
    .with_maximum_concurrency(2)
    .with_timeout_secs(120);

let runner = ChatRunner::default().with_leaf_config(cfg);
```

`LeafConfig` settings override the corresponding inherited values. Defaults are `spawn_leaf`, 300 seconds, and otherwise the parent's system prompt, iteration limit and runner tool concurrency. The leaf inherits the parent's template, eviction strategy and original tools. A name collision with an existing tool is rejected.

## What happens for one `spawn_leaf` call?

```text
Parent LLM
   -> spawn_leaf({ "task": "Summarize module X", "context": "optional guidance" })
       -> fresh AgentState (no parent chat history)
       -> inherited engine + original tool snapshot
       -> run the same internal ReAct loop
       -> return final answer only
       -> drop temporary state
   -> parent receives the answer as a tool result
```

The model-facing argument schema has a required `task` string and an optional `context` string. The optional context becomes the Leaf's dynamic context. The parent must supply whatever background the Leaf needs: it **does not inherit the parent's conversation**.

The tooling is snapshotted **before** `spawn_leaf` is added. The parent sees original tools plus Leaf; the Leaf sees only the originals, so nested Leaf spawning is not available through this mechanism.

Leaf uses `final_answer_only` in the loop to avoid returning intermediate per-round display output. This saves *parent* context, not necessarily model tokens spent inside the Leaf.

## Important boundaries

| Boundary | In 0.4.1 |
| --- | --- |
| Conversation history | Fresh `AgentState` per Leaf invocation; dropped afterward |
| Inference engine | **Shared** with the parent, not an isolated model instance |
| Tools | Inherits **all** original parent tools; no built-in allow-list |
| Recursive Leaf spawning | Structurally unavailable through the Leaf tool snapshot |
| Side effects | **Not** rolled back; a write/email tool can still act |
| Concurrency | Leaf loop limit is per invocation, **not** a global budget |
| Local llama.cpp | A shared worker/KV cache can serialize work and require re-prefill |
| Tool timeout | 300 seconds by default; timeout does not prove external cancellation |
| Persistence | Leaf history is temporary; no parent eviction callback |

Use a separate Agent/tool set for untrusted or sensitive delegation. The built-in Leaf is **not** a permission sandbox. Streaming parents can use the leaf tool, but the Leaf itself runs its internal loop synchronously and returns one final answer.

## When not to use it

Avoid Leaf when the subtask needs a durable identity, long-running checkpointing, a separate credential boundary, another inference engine selected by the parent model, or arbitrary recursive multi-agent orchestration. Those are different concerns and are not promised by this implementation.

See [Tools](/guide/tools) for timeout semantics and [Architecture](/advanced/architecture) for state and engine sharing.
