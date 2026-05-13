# 工具调用

工具就是 LLM 可以调用的 Rust 函数。你把业务逻辑暴露成工具，Ambi 帮你处理 JSON Schema 生成、参数解析、超时、重试和并行执行。

## 定义一个工具

实现 `Tool` trait：

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
            description: "查询指定城市的实时天气。".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "city": { "type": "string", "description": "城市名称" }
                },
                "required": ["city"]
            }),
            timeout_secs: Some(10),
            max_retries: Some(2),
            is_idempotent: true,
        }
    }

    async fn call(&self, args: WeatherArgs) -> Result<WeatherResult, ToolErr> {
        // 你的实现：调用 API、查询数据库等
        Ok(WeatherResult {
            temperature: 22.5,
            condition: "晴".into(),
        })
    }
}
```

## 注册工具

```rust
let agent = Agent::make(config).await?
.preamble("你是一个天气助手。")
.tool(WeatherTool)?;   // 名字冲突时返回 Err
```

当用户问"东京天气怎么样？"，LLM 可能会调用 `get_weather`。框架拦截这个调用，解析参数，执行函数，然后把结果放回对话上下文。

## 工具名唯一性

两个工具不能重名。如果注册了重复的名字，`tool()` 立即返回 `AmbiError::AgentError`。

## 用 `#[tool]` 宏

添加 `ambi-macros` crate 后，可以直接在函数上标注来减少样板代码，无需手动实现 trait。

```bash
cargo add ambi-macros
```

完整的 `#[tool]` 和 `#[agent]` 宏文档请参阅 [ambi-macros](/zh/extensions/ambi-macros)，包含参数描述、类型推断和生成代码示例等细节。

## 每个工具的配置

`ToolDefinition` 有三个重要字段：

| 字段              | 默认值        | 含义                            |
|-----------------|------------|-------------------------------|
| `timeout_secs`  | `Some(15)` | 工具最多能跑多久，超时直接打断               |
| `max_retries`   | `Some(3)`  | 超时后重试次数（仅对幂等工具生效）             |
| `is_idempotent` | `false`    | 是否可以安全重试——读操作 = 是，写操作/发邮件 = 否 |

### 为什么 is_idempotent 重要

非幂等工具**永远不会重试**。如果"发邮件"工具跑超时了，框架不会跑第二次——你不会想让用户收到两封一样的邮件。只读工具（"
查数据库"）可以安全重试。

## 工具调用的完整流程

1. LLM 输出 `[TOOL_CALL]{"name":"get_weather","args":{"city":"东京"}}[/TOOL_CALL]`
2. 解析器提取工具名和 JSON 参数
3. `ToolManager::run_tool` 查找工具，应用超时，执行
4. 如果超时且是幂等工具，重试（最多 `max_retries` 次）
5. 结果以 `Tool` 消息推进 `ChatHistory`
6. LLM 再跑一轮生成最终回复（ReAct 循环）

## 并行执行

一次 LLM 回复中的多个工具调用会并发执行。最大并发数通过 `ChatRunner` 配置（默认 5）：

```rust
use ambi::ChatRunner;

// 默认并发（5）
let runner = ChatRunner::default();


// 自定义并发限制
let runner = ChatRunner::new(3);
```

```rust
stream::iter(calls)
.map(|(name, args, id)| run_tool(name, args, id))
.buffered(runner.maximum_concurrency)
```

如果 LLM 调了三个工具，它们并行跑。一个比较慢不会阻塞其他的。

## 幽灵调用取消

流式模式下，如果客户端断开了连接，Ambi 会立即丢弃所有正在执行中的工具 future。这防止了后端资源被孤儿任务浪费。

## JSON 格式错误恢复

如果 LLM 输出了不合法的 JSON（多余的逗号、花括号没闭合），解析器会生成一个特殊的 `__format_error__` 调用。框架在下一次 LLM
请求里注入纠错提示，让模型自己修正格式。不会崩溃，给模型一次改过的机会。
