# 工具：组合能力，也要约束副作用

Tool 是模型产生的操作意图与应用业务逻辑之间的边界。Ambi 使用类型化 Rust `Tool`、JSON Schema 参数说明、参数解码、超时策略和循环内并发工具执行。

## 定义类型化工具

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
            description: "计算两个数字之和".into(),
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

此例还需在依赖中加入 `async-trait`、开启 derive 的 `serde` 和 `serde_json`。通过 `.tool(Add)?` 注册；重名工具会在注册时被拒绝。

## 超时和重试是一种策略

`ToolManager` 执行超时控制（未指定时默认 15 秒）。仅 `is_idempotent = true` 的工具允许超时自动重试；如果未配置，幂等工具默认重试次数为 3。业务逻辑明确返回错误时不重试。非幂等工具不会自动重试。

| 工具 | `is_idempotent` | 原因 |
| --- | --- | --- |
| 纯计算、只读查询 | `true` | 一般可以安全重复 |
| 发邮件、支付、外部写入 | `false` | 重新执行可能重复产生影响 |

**重要：** 异步超时取消 Rust Future，并不表示外部服务已经回滚操作。重要业务动作仍应使用业务幂等键、持久化操作状态或确认流程。

## ReAct 与并发执行

内置 `ChatRunner` 通过解析器获取模型提出的工具调用，再执行工具。同一轮模型响应中的多个工具调用可以并发，受 `ChatRunner::new(n)` 控制，默认上限为 5。它是**单次执行循环内**的并发上限，并非进程全局预算。执行结果会进入历史供模型继续推理。

```rust
let agent = Agent::make(engine_config).await?
    .tool(Add)?;
let runner = ChatRunner::new(3);
```

## 工具语法与解析

默认解析器识别特殊标记包裹的 JSON 工具调用；OpenAI 兼容模型的原生 Tool Call 会由适配层转为 Ambi 内部表示。你可以更换 `ToolCallParser` 或配置标签。详见 [工具解析器](/zh/advanced/tool-parser)。

## Leaf 中的工具

Leaf 通过向父级暴露 `spawn_leaf` 工具实现委派。但 Leaf 会继承父级**原始工具**，包括写入、网络等副作用能力；它没有内置 Allow-list 或权限沙箱。同时，Leaf 的工具快照刻意不包含自己的 `spawn_leaf`，因此不能通过该接口递归。

详见 [Leaf](/zh/guide/leaf) 和 [设计理念](/zh/advanced/design-philosophy)。
