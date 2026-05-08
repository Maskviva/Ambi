# Custom Pipeline

The pipeline is the execution loop that drives the Agent. `ChatRunner` is the default, implementing the full ReAct loop with tool handling and streaming.

## When to write a custom pipeline

- You want a simpler loop without tool support (pure chat)
- You need different error handling or retry logic
- You're building a specialized agent (e.g., chain-of-thought only, no tools)

## Implementing `Pipeline`

```rust
use ambi::agent::pipeline::Pipeline;
use ambi::agent::core::{Agent, AgentState};
use ambi::error::Result;
use ambi::ContentPart;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

struct MySimplePipeline;

impl Pipeline for MySimplePipeline {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        // Build the request from current state
        let req = /* ... */;

        // Call the engine directly
        let response = agent.llm_engine.chat(req).await?;

        // No tool handling – just return the raw text
        Ok(response)
    }

    async fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        // ... stream implementation
        unimplemented!()
    }
}
```

Note that `llm_engine` is `pub(crate)`, so you can only implement custom pipelines from within the crate, or you compose with `ChatRunner`'s public API.

## Making a pipeline that composes with ChatRunner

For most custom requirements, you don't need to replace the entire pipeline – you can customize individual parts:

- **Formatting** → custom `StreamFormatter` (see [Stream Formatter](/advanced/stream-formatter))
- **Tool parsing** → custom `ToolCallParser` (see [Tool Parser](/advanced/tool-parser))
- **Engine behavior** → custom `LLMEngineTrait` (see [Custom Engine](/advanced/custom-engine))

### Pre-built pipelines

The [`ambi-pipelines`](/extensions/ambi-pipelines) extension crate offers production-ready pipeline implementations:

| Pipeline | Description |
|---|---|
| `StandardRagPipeline` | Retrieval-Augmented Generation with token-aware context packing |
| `SelfConsistencyPipeline` | Parallel Chain-of-Thought with majority voting |
| `BfsBeamSearchPipeline` | Tree of Thoughts with BFS beam search |
| `ReflexionPipeline` | Actor-Evaluator loop with persistent critique memory |

If you still need a completely custom loop, copy `ChatRunner`'s structure and modify what you need.
