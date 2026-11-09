# ambi-macros：减少样板代码，不改变 Runtime

`ambi-macros` 是独立的过程宏 Crate。生成的代码仍实现 Ambi 普通的 Rust Trait；宏不会建立另一套 Agent 执行模型。当前仓库中的 Crate 版本为 **0.1.2**。

通过核心 Crate 的 `macro` Feature 使用重新导出的宏：

```toml
[dependencies]
ambi = { version = "0.4.1", features = ["macro"] }
serde = { version = "1", features = ["derive"] }
```

也可以直接依赖匹配版本的 `ambi-macros`。

## `#[tool]`：把 Rust 函数变成 Tool

```rust
use ambi::macros::tool;
use ambi::types::ToolErr;

/// 两个整数相加。
#[tool(name = "add", timeout = 10, idempotent)]
async fn add(a: i32, b: i32) -> Result<i32, ToolErr> {
    Ok(a + b)
}

// 宏生成 AddTool，实现 ambi::types::Tool。
// let agent = Agent::make(engine_config).await?.tool(AddTool)?;
```

支持 `name`、`description` / `desc`、`timeout` / `timeout_secs`、`idempotent` / `is_idempotent`、`max_retries` / `retries`，以及 `params(key = "description", ...)` 形式的参数描述。`Option<T>` 会生成非必填的 JSON Schema 属性。

宏生成工具仍然遵守普通 `Tool` 的超时、幂等性与重试规则。宏不能让外部副作用自动成为事务。参见 [工具](/zh/guide/tools)。

## `#[agent]`：可选的门面类型

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

这会在核心 Agent/State/Pipeline 机制上生成一个便利的门面与 Builder，并**不改变** Agent 共享能力、State 保存会话的基本边界。完整用例见 [`examples/crate_agent_in_macro.rs`](https://github.com/Maskviva/Ambi/blob/main/examples/crate_agent_in_macro.rs)，理念参见 [设计理念](/zh/advanced/design-philosophy)。
