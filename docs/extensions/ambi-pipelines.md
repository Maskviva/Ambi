# ambi-pipelines: optional execution strategies

`ambi-pipelines` keeps higher-level reasoning patterns outside Ambi's small core. Its repository source version is **0.1.1** alongside Ambi 0.4.1. These pipelines implement the `Pipeline` contract; they are **not** generic graph nodes and do not automatically compose with every other pipeline.

```toml
[dependencies]
ambi = "0.4.1"
ambi-pipelines = "0.1.1"
```

Use matching workspace path dependencies if the versions are not available from your registry.

## Included strategies

| Pipeline | How it executes | Key limitation |
| --- | --- | --- |
| `StandardRagPipeline` | Retrieves and packs documents, writes dynamic context, delegates to an inner `ChatRunner` | Mutates session dynamic context |
| `SelfConsistencyPipeline` | Runs branches and selects an aggregated answer | Streaming not implemented |
| `BfsBeamSearchPipeline` | Expands/evaluates candidate thought branches | Requires user-provided expander and evaluator |
| `ReflexionPipeline` | Runs an actor/evaluator loop, stores critiques using a KV provider | Streaming not implemented |

## Example: RAG pipeline

```rust
use ambi::agent::pipeline::Pipeline;
use ambi_pipelines::rag::StandardRagPipeline;

// my_retriever implements ambi_pipelines::rag::Retriever.
let pipeline = StandardRagPipeline::create(my_retriever)
    .max_context_tokens(4096)
    .concurrency(3);

let answer = pipeline.execute(&agent, &state, input_parts).await?;
```

The `Retriever` and `Document` abstractions belong to `ambi-pipelines`, not core `ambi`. RAG packs the retrieved text into `AgentState.dynamic_context` before using its internal ChatRunner, so the application should account for context freshness and shared-state concurrency.

## Why these are separate crates

Ambi provides the execution substrate; an application selects the reasoning strategy. The built-in examples should not impose a universal cognitive architecture on users who only need a small ReAct loop or a custom pipeline.

**Leaf availability:** The 0.4.1 Leaf feature is mounted on `ChatRunner` explicitly; these auxiliary pipelines may create internal `ChatRunner::default()` instances, which remain Leaf-free even if the Cargo feature is compiled. See [Leaf](/guide/leaf) and [Custom Pipeline](/advanced/custom-pipeline).
