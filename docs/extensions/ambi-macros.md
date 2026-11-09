# ambi-macros: reduce boilerplate, keep the same runtime

`ambi-macros` is a separate procedural macro crate. The generated code implements Ambi's ordinary Rust traits; macros do not introduce a different Agent execution model. In this repository the crate version is **0.1.2**.

Use the `macro` feature of the main crate to access the re-export:

```toml
[dependencies]
ambi = { version = "0.4.1", features = ["macro"] }
serde = { version = "1", features = ["derive"] }
```

Or depend directly on `ambi-macros` at a matching version.

## `#[tool]`: a Rust function becomes a Tool

```rust
use ambi::macros::tool;
use ambi::types::ToolErr;

/// Add two integers.
#[tool(name = "add", timeout = 10, idempotent)]
async fn add(a: i32, b: i32) -> Result<i32, ToolErr> {
    Ok(a + b)
}

// The macro generates AddTool, which implements ambi::types::Tool.
// let agent = Agent::make(engine_config).await?.tool(AddTool)?;
```

Attributes include `name`, `description` / `desc`, `timeout` / `timeout_secs`, `idempotent` / `is_idempotent`, `max_retries` / `retries`, and parameter descriptions via `params(key = "description", ...)`. `Option<T>` arguments are optional in the generated JSON Schema.

The resulting tool still follows the same timeout and idempotency semantics as a manually implemented `Tool`. A macro cannot make external side effects transactional. See [Tools](/guide/tools).

## `#[agent]`: an optional facade

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

This generates a convenience facade and builder around core Agent/State/Pipeline mechanics. It does **not** change the underlying distinction between shared `Agent` capabilities and per-session state. See [`examples/crate_agent_in_macro.rs`](https://github.com/Maskviva/Ambi/blob/main/examples/crate_agent_in_macro.rs) for a complete example and [Design philosophy](/advanced/design-philosophy) for the architectural rationale.
