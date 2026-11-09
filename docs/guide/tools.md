# Tools: compose capabilities, control effects

Tools are the boundary between model-generated intent and application operations. Ambi exposes typed Rust `Tool` implementations with JSON Schema descriptions, argument decoding, timeout policy, and per-loop concurrent execution.

## Define a typed tool

```rust
use ambi::types::{Tool, ToolDefinition, ToolErr};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CalcArgs { a: f64, b: f64 }

#[derive(Serialize)]
struct CalcResult { result: f64 }

struct Add;

#[async_trait]
impl Tool for Add {
    const NAME: &'static str = "add";
    type Args = CalcArgs;
    type Output = CalcResult;

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Add two numbers".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": { "a": { "type": "number" }, "b": { "type": "number" } },
                "required": ["a", "b"]
            }),
            timeout_secs: Some(10),
            max_retries: Some(1),
            is_idempotent: true,
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr> {
        Ok(CalcResult { result: args.a + args.b })
    }
}
```

Add `async-trait`, `serde` (derive enabled) and `serde_json` alongside `ambi` for this example. Register with `.tool(Add)?`; duplicate tool names fail during registration.

## Timeout and retries are explicit *policies*

`ToolManager` enforces a timeout (15 seconds if unspecified). A timed-out tool may be retried only if `is_idempotent` is `true`, using `max_retries` (3 if unspecified for idempotent tools). A tool returning a business error is not retried. A non-idempotent tool is never automatically retried.

| Example | `is_idempotent` | Why |
| --- | --- | --- |
| Pure arithmetic or read-only lookup | `true` | Repeat is generally safe |
| Sending an email, payment, external write | `false` | Retry may duplicate an effect |

**Important:** an async timeout cancels the awaited Rust future; it cannot certify that a request already received by an external service was undone. For consequential actions, use domain-level idempotency keys, durable status records, or confirmation controls.

## ReAct and parallel calls

The built-in `ChatRunner` parses model tool calls using the registered parser and invokes tools. Multiple calls in one model response can run concurrently up to `ChatRunner::new(n)`'s limit (default `5`). This is a **per-loop** limit, not a process-wide budget. After tool results are appended, the model can run another iteration.

```rust
let agent = Agent::make(engine_config).await?
    .tool(Add)?;
let runner = ChatRunner::new(3);
```

## Tool syntax and parsing

The default parser uses special tags around JSON tool-call data, while OpenAI-compatible model responses are translated into Ambi's internal tool-call representation. You can replace the `ToolCallParser` or configure tags; see [Tool Parser](/advanced/tool-parser).

## Tools inside Leaf

The built-in Leaf delegates by exposing `spawn_leaf` as another tool. However, it inherits the parent's **original** tools, including writing or network side effects. It does not provide an allow-list or permission sandbox. The Leaf's snapshot deliberately omits its own `spawn_leaf` tool, preventing recursion through that interface.

See [Leaf](/guide/leaf) for the trade-off and [Design philosophy](/advanced/design-philosophy) for why delegation is a tool instead of another permanent Agent entity.
