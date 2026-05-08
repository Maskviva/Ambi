# ambi-macros

`ambi-macros` is a procedural macro crate that eliminates boilerplate when defining tools and agents. It is shipped as a separate sub-crate and exposed through the `macro` feature flag of the main `ambi` crate.

```toml
[dependencies]
ambi = { version = "0.3", features = ["openai-api", "macro"] }
```

---

## `#[tool]` — Auto-implement `Tool` for any function

Instead of manually implementing the `Tool` trait (defining `Args`, `Output`, `ToolDefinition`, and async `call()`), annotate any `async fn` with `#[tool]` and the macro generates everything for you.

### Basic usage

```rust
use ambi::macros::tool;
use ambi::types::ToolErr;

#[tool(name = "search_docs", description = "Search documentation")]
async fn search_docs(query: String) -> Result<String, ToolErr> {
    Ok(format!("Results for: {}", query))
}
```

The macro generates:
- An arguments struct (`SearchDocsArgs`) with `#[derive(Deserialize)]`
- A tool struct (`SearchDocsTool`) implementing `Tool`
- A `ToolDefinition` with auto-inferred JSON Schema from Rust types
- Registration convenience via `SearchDocsTool`

### Supported attributes

| Attribute      | Short alias  | Default | Description |
|----------------|-------------|---------|-------------|
| `name`         | —           | Function name | The tool name exposed to the LLM |
| `description`  | `desc`      | Doc comment  | What the tool does (LLM-facing) |
| `timeout`      | `timeout_secs` | `None` | Max wall-clock seconds before abort |
| `idempotent`   | `is_idempotent` | `false` | Whether the tool is safe to retry on timeout |
| `retries`      | `max_retries` | `None` | Number of retries on timeout (idempotent only) |
| `params`       | —           | Empty | Per-parameter LLM-facing descriptions |

### Parameter descriptions with `params(...)`

Use `params` to provide richer routing hints for the model:

```rust
#[tool(name = "check_city_weather", description = "Get current weather for a city")]
async fn get_weather(
    city: String,
    days: Option<u32>,
) -> Result<WeatherOutput, ToolErr> {
    // ...
}
```

The `params(...)` attribute provides descriptions for each parameter:

```rust
#[tool(
    name = "check_city_weather",
    description = "Get current weather for a city",
    params = {
        "city": "Name of the city to query",
        "days": "Optional forecast days"
    }
)]
async fn get_weather(city: String, days: Option<u32>) -> Result<WeatherOutput, ToolErr> {
    // ...
}
```

### Type inference

The macro maps Rust types to JSON Schema types automatically:

| Rust type | JSON Schema |
|-----------|-------------|
| `String`, `&str`, `char` | `"string"` |
| `i8`–`i64`, `u8`–`u64`, `isize`, `usize` | `"integer"` |
| `f32`, `f64` | `"number"` |
| `bool` | `"boolean"` |
| `Vec<T>`, `HashSet<T>` | `"array"` |
| `HashMap<K,V>`, `serde_json::Value` | `"object"` |
| `Option<T>` | Inferred inner type, **not required** |

### How to register a macro-defined tool

The macro generates a struct named `{PascalCaseFn}Tool`. Register it with `.tool()`:

```rust
let agent = Agent::make(config).await?
    .preamble("You are a weather assistant.")
    .tool(GetWeatherTool)?;
```

### Generated code example

For a function `async fn search_docs(query: String) -> Result<String, ToolErr>`, the macro generates approximately:

```rust
#[derive(::serde::Deserialize)]
pub struct SearchDocsArgs {
    pub query: String,
}

pub struct SearchDocsTool;

#[async_trait::async_trait]
impl ::ambi::types::Tool for SearchDocsTool {
    const NAME: &'static str = "search_docs";
    type Args = SearchDocsArgs;
    type Output = String;

    fn definition(&self) -> ::ambi::types::ToolDefinition { /* ... */ }
    async fn call(&self, args: Self::Args) -> Result<String, ToolErr> {
        search_docs(args.query).await
    }
}
```

---

## `#[agent]` — Auto-generate Agent facade

The `#[agent]` macro generates a complete Agent wrapper struct with a fluent builder, removing all manual wiring.

```rust
use ambi::macros::{agent, tool};
use ambi::types::ToolErr;

#[tool(name = "add", timeout = 10, idempotent)]
async fn add(a: i32, b: i32) -> Result<i32, ToolErr> {
    Ok(a + b)
}

#[agent(tools = [AddTool])]
pub struct DevAgent;
```

### What the macro generates

**1. Facade struct** — bundles `Agent`, `AgentState`, and the pipeline:

```rust
pub struct DevAgent {
    pub agent: Agent,
    pub state: Arc<RwLock<AgentState>>,
    runner: ChatRunner,
}
```

**2. Builder** — fluent construction:

```rust
let assistant = DevAgent::builder(engine_config)
    .preamble("You are an intelligent assistant.")
    .session_id("my-session")
    .build()
    .await?;
```

**3. Convenience methods** — direct access to common operations:

```rust
// Chat
let reply = assistant.chat("What is 114514 plus 8080?").await?;

// Stream
let mut stream = assistant.chat_stream("Tell me a story").await?;

// Multimodal
let reply = assistant.execute(vec![
    ContentPart::Text { text: "What's in this image?" },
    ContentPart::Image { base64: image_str },
]).await?;

// Context management
assistant.set_dynamic_context("Relevant docs: ...").await;
assistant.append_dynamic_context("User locale: zh-CN").await;
assistant.clear_dynamic_context().await;
assistant.clear_history().await;
```

### Supported attributes

| Attribute  | Default | Description |
|------------|---------|-------------|
| `tools = [...]` | `[]` | List of tool structs to register |
| `pipeline = ...` | `ChatRunner` | Custom pipeline implementation |

### Custom pipeline example

```rust
#[agent(tools = [AddTool], pipeline = MyCustomPipeline)]
pub struct DevAgent;
```

Then build with a custom runner:

```rust
let assistant = DevAgent::builder(engine_config, MyCustomPipeline)
    .preamble("You are helpful.")
    .build()
    .await?;
```

---

## Feature flag

Add `"macro"` to your `ambi` dependency features:

```toml
[dependencies]
ambi = { version = "0.3", features = ["openai-api", "macro"] }
```

The `macro` feature re-exports `ambi_macros` as `ambi::macros`, so you can use:

```rust
use ambi::macros::tool;
use ambi::macros::agent;
```

Or equivalently:

```rust
use ambi::macros::{tool, agent};
```
