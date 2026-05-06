# Tools

A tool is a Rust function that the LLM can decide to call. You expose your business logic as tools, and Ambi handles the wiring: JSON schema generation, argument parsing, timeout, retry, and parallel execution.

## Defining a tool

Implement the `Tool` trait:

```rust
use ambi::{Tool, ToolDefinition, ToolErr};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

#[derive(Deserialize)]
struct WeatherArgs {
    city: String,
}

#[derive(Serialize)]
struct WeatherResult {
    temperature: f64,
    condition: String,
}

struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    const NAME: &'static str = "get_weather";

    type Args = WeatherArgs;
    type Output = WeatherResult;

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "get_weather".into(),
            description: "Get the current weather for a city.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "city": { "type": "string", "description": "City name" }
                },
                "required": ["city"]
            }),
            timeout_secs: Some(10),
            max_retries: Some(2),
            is_idempotent: true,
        }
    }

    async fn call(&self, args: WeatherArgs) -> Result<WeatherResult, ToolErr> {
        // Your actual implementation: call an API, query a DB, etc.
        Ok(WeatherResult {
            temperature: 22.5,
            condition: "Sunny".into(),
        })
    }
}
```

## Registering a tool

```rust
let agent = Agent::make(config).await?
    .preamble("You are a weather assistant.")
    .tool(WeatherTool)?;   // <-- returns Err if name conflicts
```

Now when the user asks "What's the weather in Tokyo?", the LLM may invoke `get_weather`. The framework catches the tool call, parses the arguments, runs your function, and feeds the result back into the conversation.

### Tool name uniqueness

Tool names must be unique. If you register two tools with the same name, `tool()` returns `AmbiError::AgentError` immediately (fail-fast).

## Using the `#[tool]` macro

If you enable the `macro` feature, you can reduce boilerplate:

```toml
ambi = { version = "0.3", features = ["openai-api", "macro"] }
```

```rust
use ambi::tool;

#[tool(
    name = "search_docs",
    description = "Search documentation",
    params = {
        "query": {
            "type": "string",
            "description": "Search term"
        }
    }
)]
async fn search_docs(query: String) -> Result<String, ToolErr> {
    Ok(format!("Results for: {}", query))
}
```

The `params(...)` attribute lets you inject LLM-facing descriptions directly into function arguments,
providing richer routing hints to the model. The macro generates the `Tool` implementation, the
argument struct, and the `ToolDefinition` automatically.

## Per-tool configuration

Every `ToolDefinition` has three important fields:

| Field | Default | Meaning |
|-------|---------|---------|
| `timeout_secs` | `Some(15)` | Max wall-clock time before the tool is aborted |
| `max_retries` | `Some(3)` | Number of retries on timeout (only applies if idempotent) |
| `is_idempotent` | `false` | Whether it's safe to retry – read operations = yes, writes/emails = no |

### Why `is_idempotent` matters

Non-idempotent tools are **never retried**. If a "send email" tool times out after 10 seconds, the framework will not run it again – you don't want duplicate emails. Read-only tools like "search database" can retry safely.

## What happens when a tool is called

1. LLM outputs `[TOOL_CALL]{"name":"get_weather","args":{"city":"Tokyo"}}[/TOOL_CALL]`
2. The parser extracts the tool name and JSON args
3. `ToolManager::run_tool` looks up the tool, applies timeout, runs it
4. If it times out and is idempotent, it retries (up to `max_retries` times)
5. The result is pushed into `ChatHistory` as a `Tool` message
6. The LLM gets another turn to produce a final answer (ReAct loop)

## Parallel execution

All tool calls from a single LLM response run concurrently. The maximum concurrency is configured
on `ChatRunner` (defaults to 5 via `ChatRunner::default()`):

```rust
use ambi::ChatRunner;

// Default concurrency (5)
let runner = ChatRunner::default();

// Custom concurrency limit
let runner = ChatRunner::new(3);
```

```rust
// pseudocode from tool_handler.rs
stream::iter(calls)
    .map(|(name, args, id)| run_tool(name, args, id))
    .buffered(runner.maximum_concurrency)
```

If the LLM calls three tools, they execute in parallel. If one of them is slow, the others are not blocked.

## Ghost call cancellation

When streaming, if the client disconnects mid-tool-execution, Ambi immediately discards any pending tool futures. This prevents orphaned background operations.

## Error recovery for malformed JSON

If the LLM produces invalid JSON (trailing comma, unclosed brace), the parser emits a special `__format_error__` call. The framework injects a correction prompt into the next LLM turn, asking it to fix the format. This avoids crashes and gives the model a chance to self-correct.
