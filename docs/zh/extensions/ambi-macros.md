# ambi-macros

`ambi-macros` 是 Ambi 的过程宏子库，用于消除定义工具和 Agent 时的样板代码。通过主库的 `macro` 特性标志启用。

```toml
[dependencies]
ambi = { version = "0.3", features = ["openai-api", "macro"] }
```

---

## `#[tool]` — 自动为任意函数实现 `Tool`

不用手动实现 `Tool` trait（定义 `Args`、`Output`、`ToolDefinition` 和异步 `call()`），只需在任意 `async fn` 上标注 `#[tool]`，宏会为你生成所有代码。

### 基本用法

```rust
use ambi::macros::tool;
use ambi::types::ToolErr;

#[tool(name = "search_docs", description = "搜索文档")]
async fn search_docs(query: String) -> Result<String, ToolErr> {
    Ok(format!("搜索结果：{}", query))
}
```

宏会自动生成：
- 参数结构体 `SearchDocsArgs`（带 `#[derive(Deserialize)]`）
- 工具结构体 `SearchDocsTool`（实现 `Tool` trait）
- 从 Rust 类型自动推断 JSON Schema 的 `ToolDefinition`
- 注册入口 `SearchDocsTool`

### 支持的属性

| 属性 | 短别名 | 默认值 | 说明 |
|------|--------|--------|------|
| `name` | — | 函数名 | 暴露给 LLM 的工具名称 |
| `description` | `desc` | 文档注释 | 工具功能说明（面向 LLM） |
| `timeout` | `timeout_secs` | `None` | 工具超时秒数 |
| `idempotent` | `is_idempotent` | `false` | 超时后是否可以安全重试 |
| `retries` | `max_retries` | `None` | 超时重试次数（仅幂等工具） |
| `params` | — | 空 | 每个参数的 LLM 面向描述 |

### 参数描述 `params(...)`

用 `params` 为模型提供更丰富的路由提示：

```rust
#[tool(
    name = "check_city_weather",
    description = "获取指定城市的实时天气",
    params = {
        "city": "要查询的城市名称",
        "days": "可选预报天数"
    }
)]
async fn get_weather(city: String, days: Option<u32>) -> Result<WeatherOutput, ToolErr> {
    // ...
}
```

### 类型推断

宏自动将 Rust 类型映射为 JSON Schema 类型：

| Rust 类型 | JSON Schema |
|-----------|-------------|
| `String`, `&str`, `char` | `"string"` |
| `i8`–`i64`, `u8`–`u64`, `isize`, `usize` | `"integer"` |
| `f32`, `f64` | `"number"` |
| `bool` | `"boolean"` |
| `Vec<T>`, `HashSet<T>` | `"array"` |
| `HashMap<K,V>`, `serde_json::Value` | `"object"` |
| `Option<T>` | 推断内部类型，**非必填** |

### 注册宏定义的工具

宏生成的结构体名为 `{帕斯卡命名函数}Tool`。通过 `.tool()` 注册：

```rust
let agent = Agent::make(config).await?
    .preamble("你是一个天气助手。")
    .tool(GetWeatherTool)?;
```

### 生成的代码示例

对于函数 `async fn search_docs(query: String) -> Result<String, ToolErr>`，宏大致生成：

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

## `#[agent]` — 自动生成 Agent 门面

`#[agent]` 宏生成一个完整的 Agent 包装结构体，附带流式构建器，消除所有手动接线。

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

### 宏生成的内容

**1. 门面结构体** — 整合 `Agent`、`AgentState` 和管道：

```rust
pub struct DevAgent {
    pub agent: Agent,
    pub state: Arc<RwLock<AgentState>>,
    runner: ChatRunner,
}
```

**2. 构建器** — 流式构造：

```rust
let assistant = DevAgent::builder(engine_config)
    .preamble("你是一个智能助手。")
    .session_id("my-session")
    .build()
    .await?;
```

**3. 便捷方法** — 直接调用常见操作：

```rust
// 对话
let reply = assistant.chat("114514 加 8080 等于多少？").await?;

// 流式
let mut stream = assistant.chat_stream("讲个故事").await?;

// 多模态
let reply = assistant.execute(vec![
    ContentPart::Text { text: "这张图里有什么？" },
    ContentPart::Image { base64: image_str },
]).await?;

// 上下文管理
assistant.set_dynamic_context("相关文档：...").await;
assistant.append_dynamic_context("用户语言：zh-CN").await;
assistant.clear_dynamic_context().await;
assistant.clear_history().await;
```

### 支持的属性

| 属性 | 默认值 | 说明 |
|------|--------|------|
| `tools = [...]` | `[]` | 要注册的工具结构体列表 |
| `pipeline = ...` | `ChatRunner` | 自定义管道实现 |

### 自定义管道示例

```rust
#[agent(tools = [AddTool], pipeline = MyCustomPipeline)]
pub struct DevAgent;
```

用自定义运行器构建：

```rust
let assistant = DevAgent::builder(engine_config, MyCustomPipeline)
    .preamble("你是一个助手。")
    .build()
    .await?;
```

---

## 特性标志

在 `ambi` 依赖特性中添加 `"macro"`：

```toml
[dependencies]
ambi = { version = "0.3", features = ["openai-api", "macro"] }
```

启用后，`ambi::macros` 模块会重导出所有过程宏：

```rust
use ambi::macros::tool;
use ambi::macros::agent;
```

或等效：

```rust
use ambi::macros::{tool, agent};
```
