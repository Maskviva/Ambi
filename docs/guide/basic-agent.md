# Agent and AgentState: reuse without shared conversations

The `Agent` holds *capabilities*; `AgentState` holds *a conversation*. Keeping them separate is more important than adding a new Agent subtype for every workflow.

## Reuse one Agent

```rust
let agent = Agent::make(engine_config).await?
    .preamble("You are a concise assistant.")
    .max_iterations(8);

let runner = ChatRunner::default();
let alice = AgentState::new_shared("conversation-alice");
let bob = AgentState::new_shared("conversation-bob");

let a = runner.chat(&agent, &alice, "Hello").await?;
let b = runner.chat(&agent, &bob, "Hello").await?;
```

These two states have different histories. `Agent` itself is cheap to clone because it shares most resources with `Arc`. Use distinct session IDs when possible; local `llama.cpp` uses them when deciding whether to reuse cached inference state.

## Templates and prompts

`Agent::preamble(&str)` sets the reusable system prompt. `.template(ChatTemplateType::Chatml)` chooses a built-in chat template; models may require a different format. You can also provide your own `ChatTemplate`.

```rust
use ambi::types::ChatTemplateType;
let agent = Agent::make(engine_config).await?
    .template(ChatTemplateType::Chatml)
    .preamble("Answer with evidence and disclose uncertainty.");
```

## Dynamic context belongs to the session

```rust
{
    let mut session = alice.write().await;
    session.set_dynamic_context("Reference: project overview v3");
    session.append_dynamic_context("Locale: en-US");
}

// Remove temporary session context when it becomes stale.
alice.write().await.clear_dynamic_context();
```

Dynamic context is appended to the model's effective system context, not normal chat history. It is not automatically pruned by FIFO history eviction. Treat it as an explicit application responsibility and refresh it when retrieved information changes.

## Fork a state, or start fresh

`AgentState::fork()` clones the state while sharing underlying history message contents through `Arc`. This is useful for branching experiments such as ToT. `AgentState::new_shared()` creates a new independent conversation.

To clear history:

```rust
let mut session = alice.write().await;
ChatRunner::clear_history(&agent, &mut session);
```

**Caution:** `clear_history` also calls `engine.reset_context()` on the agent's shared engine. On a shared local engine, that reset is not scoped only to the current session.

## Concurrency is the application's decision

`Arc<RwLock<AgentState>>` prevents unsynchronized individual Rust memory accesses. It does **not** guarantee entire chat requests on one state will execute serially. If ordering matters, put an application-level session mutex or queue around calls using that state. Distinct states can share the same Agent.

## Errors

Public operations return `Result<_, AmbiError>`. Handle engine errors, tool failures, pipeline errors and `MaxIterationsReached` explicitly when relevant. A failed run may already have performed irreversible tool actions; do not assume error means nothing happened.

Next: [Architecture](/advanced/architecture) · [Tools](/guide/tools) · [Leaf](/guide/leaf).
