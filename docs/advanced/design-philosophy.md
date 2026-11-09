# Design philosophy: a small runtime, an open ceiling

Ambi is a **small, composable AI Agent Runtime for Rust**. It provides mechanisms for model inference, tool execution, context management, and running an agent. It deliberately does **not** prescribe a universal way for agents to reason, plan, or cooperate.

This is a design contract, not a claim that any collection of traits makes arbitrary workflows possible automatically. The public APIs remain specific and sometimes impose constraints. The point is to keep the core understandable and make new behavior possible through composition wherever the existing contracts allow it.

## Composition over inheritance

Ambi avoids a hierarchy of `BaseAgent` / `WorkerAgent` / `SupervisorAgent`. Instead, an application assembles capabilities:

| Concept | Responsibility | Source of variation |
| --- | --- | --- |
| `Agent` | Reusable engine, configuration, tools, parser and formatter | Builder methods and shared components |
| `AgentState` | A session's history and dynamic context | One state per conversation, or explicitly forked states |
| `Pipeline` | How a request is executed | Implement your own pipeline |
| `LLMEngineTrait` | How a model produces an answer or a stream | Built-in or custom backend |
| `Tool` | A typed operation available to the model | Application-defined Rust implementations |
| `ToolCallParser`, `StreamFormatter`, `TokenizerTrait` | Protocol and presentation boundaries | Replaceable implementations |

**Ambi's built-in examples set the floor, not the ceiling.** A third-party `Pipeline` can introduce an execution strategy that Ambi's authors never anticipated, without requiring that strategy to become part of the core crate.

A distinction matters: **replaceability is not the same as arbitrary pipeline composition**. For instance, the current `StandardRagPipeline` owns an internal `ChatRunner`; it is not a generic `Rag<P>` wrapper. Ambi does not claim to provide a universal graph execution engine.

## Tree, branch, leaf: an execution metaphor

```text
                               Agent
                 reusable engine, tools, configuration
                                 |
                  +--------------+---------------+
                  |                              |
               ChatRunner                    Custom Pipeline
             (ReAct branch)                 (another branch)
                  |
         optional spawn_leaf tool
                  |
             +----+----+
             |         |
          Leaf A      Leaf B
        fresh state  fresh state
             |         |
       final answer  final answer

         AgentState is supplied separately to a pipeline.
```

**Agent is the tree; Pipeline is the branch; Leaf is a leaf.** This is **not** a Rust inheritance tree or a requirement that every pipeline grow leaves. Different branches can have different capabilities, and downstream developers can graft in their own behavior through traits.

Why *Leaf* rather than *Sub-agent*? A delegated task does not need a second persistent `Agent` identity simply because a parent invoked it. In Ambi 0.4.1, a Leaf is a short-lived **execution**, attached as a tool to `ChatRunner`, with an isolated temporary conversation state. It returns a compact result and disappears. Its value is that intermediate task context does not automatically become parent history.

This is **lifecycle and context isolation**, not a claim of side-effect isolation: the built-in Leaf inherits the parent's registered tools and shares its engine. See [Leaf task delegation](/guide/leaf).

## Mechanism belongs in the runtime; policy belongs with the application

The runtime has concrete duties: invoke model backends, honor tool timeout/retry policies, handle streaming, account for conversation history, and expose useful error paths. The application chooses system prompts, execution strategy, tool authorization, storage and business semantics.

This line also explains why Ambi keeps `ambi-pipelines`, `ambi-memory` and `ambi-macros` as separate crates. Advanced workflows should not require everyone to pay for, or adopt, one cognitive architecture.

A small public API should not hide important operational caveats:

- Token accounting used for history eviction is **approximate** and is not a universal guarantee against provider context errors.
- `AgentState` is distinct from `Agent`, but passing the **same mutable state** into concurrent runs does not by itself serialize whole conversations.
- A tool future timing out or being cancelled does **not** prove an external side effect was rolled back.
- Shared local `llama.cpp` engine state can make concurrent Leaf calls contend for one worker and KV cache.

Read [Architecture](/advanced/architecture) for the code-level boundaries and [Context eviction](/advanced/context-eviction) for operational limits.

## A rule for adding abstractions

1. Try composing existing engine, pipeline, state and tool mechanisms first.
2. Add a new core abstraction only when a real use case cannot be expressed with existing contracts.
3. Do not make an optional feature silently change the behavior of existing pipelines.
4. Keep safety and lifecycle guarantees explicit, especially when external tools have side effects.

Leaf follows these rules in 0.4.1: it is a Cargo feature **and** a `ChatRunner` opt-in; it reuses the existing tool mechanism and ReAct loop; and it creates no persistent agent registry. Whether another pipeline needs a different kind of disposable task is a decision for its own author.

## What Ambi is not trying to be

Ambi does not aspire to bundle every planning method, role hierarchy, workflow graph, durable orchestrator, permission system, or memory database. A small runtime can support implementations of those ideas, but **supporting an extension** and **shipping a built-in product** are different commitments.

The design target is therefore **a stable, comprehensible execution substrate**—not the longest feature list.
