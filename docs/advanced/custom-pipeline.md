# Custom Pipeline: define execution, not a new Agent class

`Pipeline` defines *what happens when a request arrives*. The built-in `ChatRunner` implements a ReAct-style model/tool loop, but a pipeline is not required to use that strategy. Keeping it separate from `Agent` is one of Ambi's central composition boundaries.

Read [Design philosophy](/advanced/design-philosophy) and [Architecture](/advanced/architecture) first.

## The public contract

On native targets, `Pipeline` has two required methods:

```rust
use ambi::agent::pipeline::Pipeline;
// execute(&self, &Agent, &Arc<RwLock<AgentState>>, Vec<ContentPart>) -> Future<Result<String>>
// execute_stream(...) -> Future<Result<Pin<Box<ReceiverStream<Result<String>>>>>>
```

The actual trait uses `impl Future` return types and is `Send + Sync` on native targets. Both methods are required; returning a `PipelineError` for an unsupported mode is a legitimate design choice (the built-in Reflexion pipeline does this for streaming).

## A minimal pipeline that composes the built-in runner

This example uses a wrapper rather than accessing private `Agent` internals:

```rust
use ambi::agent::pipeline::Pipeline;
use ambi::error::Result;
use ambi::{Agent, AgentState, ChatRunner, ContentPart};
use std::{pin::Pin, sync::Arc};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

struct MyPipeline {
    inner: ChatRunner,
}

impl Pipeline for MyPipeline {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        // Add your pre/post-execution policy here.
        self.inner.execute(agent, state, input).await
    }

    async fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        self.inner.execute_stream(agent, state, input).await
    }
}
```

For this example, add `tokio-stream = "0.1"` to your dependencies. Construct `MyPipeline { inner: ChatRunner::default() }`, then call `pipeline.execute(...)`; or use `ChatRunner` directly if you do not need another policy layer.

## Build a different loop

A custom pipeline can call `agent.llm_engine()` to access the shared `Arc<LLMEngine>`. It can create `LLMRequest` values and choose its own state and tool orchestration. Because the contract is explicit, implementing a truly different loop requires you to take responsibility for history updates, context construction, streaming and tool policy. Ambi does **not** expose `ChatRunner::run_loop` as a public reusable primitive.

**Do not assume arbitrary pipeline composition is already implemented.** For example, `StandardRagPipeline` and `ReflexionPipeline` currently hold an internal `ChatRunner`, rather than accepting any `P: Pipeline`. You can compose around the public interface yourself, but Ambi is not a generic DAG scheduler.

## Why Leaf lives on ChatRunner

The 0.4.1 Leaf uses `ChatRunner`'s internal ReAct loop and is added as a tool only when `.with_leaf()` or `.with_leaf_config(...)` is called. Merely enabling its Cargo feature does not change the default runner or pipelines that construct it internally.

A custom pipeline is free to implement a *different* temporary-task strategy. It does not need to implement a global `Leaf` trait. See [Leaf](/guide/leaf).

## Extension checklist

- Decide who owns shared configuration and who owns session state.
- Define streaming behavior deliberately, even if it is unsupported.
- Do not mutate the same session from overlapping runs unless you coordinate them.
- Ensure tool permissions, retries and external side effects are application-appropriate.
- Keep internal execution details out of the parent history unless they are needed.

Explore [`ambi-pipelines`](/extensions/ambi-pipelines) for concrete RAG, CoT, ToT and Reflexion strategies.
