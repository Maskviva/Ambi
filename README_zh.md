# Ambi 🦀

---

<p align="center">
  <a href="https://spdx.org/licenses/Apache-2.0.html"><img src="https://img.shields.io/badge/License-Apache%202.0-blue" alt="License: GPL v3"></a>
  <a href="https://github.com/maskviva/ambi"><img alt="github" src="https://img.shields.io/badge/github-maskviva/Ambi-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20"></a>
  <a href="https://crates.io/crates/ambi"><img alt="crates.io" src="https://img.shields.io/crates/v/ambi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20"></a>    
  <a href="https://docs.rs/ambi"><img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-ambi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20"></a><br/>
 [<a href="./README_zh.md">中文(简体)</a>] | [<a href="./README.md">English</a>] 
</p>

一个完全由 Rust 构建的灵活、高度可定制的 AI Agent 框架。以最少的样板代码、特征优先设计和零成本抽象，助你打造生产级智能体。

- **双引擎架构** — 在本地推理（基于 `llama.cpp` 并支持 GPU 加速）与云端 API（兼容 OpenAI 接口）之间无缝切换，无需改动 Agent
  代码。
- **高级工具系统** — 并行多工具执行、单工具独立超时与重试、从 Rust 结构体自动生成 JSON Schema。
- **智能上下文管理** — 保留对话逻辑的安全驱逐算法，防止令牌溢出，同时让 Agent 保持专注。
- **原生 Rust** — 内存安全、全面异步、最少依赖、编译迅速。

<br>

## 学习资源

学习 Ambi 的最佳方式是动手写一个 Agent。 [`examples/`](https://github.com/maskviva/ambi/tree/main/examples)
目录提供了完整的可运行示例，涵盖基础聊天、自定义工具、本地 GPU 推理、流式响应以及多工具并行执行。

<br>

## 安装

将以下内容添加到你的 `Cargo.toml`：

```toml
[dependencies]
ambi = "0.2"
```

仅使用云端 API（编译更快，无需 `llama.cpp` 依赖）：

```toml
ambi = { version = "0.2", default-features = false, features = ["openai-api"] }
```

<br>

## 快速开始

```rust
use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig, Message};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 选择引擎配置
    let config = LLMEngineConfig::OpenAI(ambi::OpenAIEngineConfig {
        api_key: std::env::var("OPENAI_API_KEY")?,
        base_url: "https://api.openai.com/v1".into(),
        model_name: "gpt-4o".into(),
        temp: 0.7,
        top_p: 0.95,
    });

    // 2. 构建 Agent（仅需 5 行代码！）
    let agent = Agent::make(config).await?
        .preamble("你是一个乐于助人的助手。")
        .template(ambi::ChatTemplateType::Chatml);

    // 3. 创建共享状态
    let state = Arc::new(Mutex::new(AgentState::new()));

    // 4. 运行聊天流水线
    let runner = ChatRunner;
    let response = runner.chat(&agent, &state, "你好，世界！").await?;
    println!("{}", response);

    Ok(())
}
```

<br>

## 使用本地推理

启用 `llama-cpp` 特性，并可叠加 GPU 后端：

```toml
ambi = { version = "0.2", features = ["llama-cpp", "cuda"] }
```

然后切换引擎配置：

```rust
let config = LLMEngineConfig::Llama(ambi::LlamaEngineConfig {
model_path: "./models/llama-3-8b.gguf".into(),
max_tokens:  4096,
buffer_size: 32,
use_gpu:     true,
n_gpu_layers: 100,
n_ctx:       8192,
n_tokens:    512,
n_seq_max:   1,
penalty_last_n:   64,
penalty_repeat:   1.1,
penalty_freq:     0.0,
penalty_present:  0.0,
temp:       0.7,
top_p:      0.9,
seed:       42,
min_keep:   1,
});
```

<br>

## 添加自定义工具

实现 `Tool` 特征即可定义工具，Ambi 会自动生成 JSON Schema。

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
            description: "获取指定城市的实时天气".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "城市名称"
                    }
                },
                "required": ["city"]
            }),
            timeout_secs: Some(10),
            max_retries: Some(2),
            is_idempotent: true,
        }
    }

    async fn call(&self, args: WeatherArgs) -> Result<WeatherResult, ToolErr> {
        // 在此实现你的工具逻辑
        Ok(WeatherResult {
            temperature: 22.5,
            condition: "晴朗".into(),
        })
    }
}
```

将工具挂载至 Agent：

```rust
let agent = Agent::make(config).await?
.preamble("你是一位天气助手。")
.tool(WeatherTool) ?;
```

此时当用户询问天气时，Agent 会自动调用 `get_weather` 工具。Ambi 替你处理重试、超时以及并行执行。

<br>

## 流式响应

```rust
use futures::StreamExt;

let mut stream = runner.chat_stream( & agent, & state, "给我讲个故事").await?;
while let Some(chunk) = stream.next().await {
match chunk {
Ok(text) => print ! ("{}", text),
Err(e) => eprintln !("流错误：{}", e),
}
}
```

<br>

## 上下文驱逐

当令牌预算超出时，Ambi 的上下文管理会自动驱逐旧消息。你也可以精细调整策略：

```rust
let agent = Agent::make(config).await?
.with_eviction_strategy(
2,    // 至少保留前 2 条消息
6,    // 至少保留最近 6 条消息
3000, // 触发驱逐的安全令牌上限
);
```

你还可以注册回调来处理被驱逐的消息（例如持久化存储）：

```rust
let agent = Agent::make(config).await?
.on_evict( | evicted| {
for msg in evicted {
// 存档或记录消息
}
});
```

<br>

## 自定义工具调用解析器

默认使用 `[TOOL_CALL] ... [/TOOL_CALL]` 标签。你可以提供自己的解析器：

```rust
use ambi::tool::{ToolCallParser, DefaultToolParser};

struct MyParser;

impl ToolCallParser for MyParser {
    fn format_instruction(&self, tools_json: &str) -> String {
        // 指导模型如何调用工具
        format!("可用工具：{}", tools_json)
    }

    fn parse(&self, text: &str) -> Vec<(String, serde_json::Value)> {
        // 从模型输出中提取工具调用
        vec![]
    }

    fn create_stream_formatter(&self) -> Box<dyn ambi::tool::StreamFormatter> {
        Box::new(ambi::agent::core::formatter::PassThroughFormatter)
    }
}

let agent = Agent::make(config).await?
.with_tool_parser(MyParser);
```

<br>

## 错误处理

Ambi 通过 `thiserror` 提供了清晰可操作的错误类型：

```rust
pub enum AmbiError {
    EngineError(String),
    AgentError(String),
    ToolError(String),
    ContextError(String),
    PipelineError(String),
    MaxIterationsReached(usize),
    Other(anyhow::Error),
}
```

所有公开 API 均返回 `Result<T, AmbiError>`，便于模式匹配或向上传播错误。

<br>

## 测试

Ambi 包含了全面的单元测试和集成测试。建议在开发过程中运行 `cargo test`。 测试 Agent 时，可使用模拟引擎来避免真实 API 调用：

```rust
struct MockEngine;
#[async_trait]
impl LLMEngineTrait for MockEngine {
    async fn chat(&self, _: LLMRequest) -> Result<String> {
        Ok("你好，我是模拟器。".into())
    }
    // ...
}

let agent = Agent::with_custom_engine(Box::new(MockEngine)) ?;
```

<br>

## 特性标志

Ambi 利用 Cargo 特性来保持快速编译：

- **`openai-api`** （默认启用） — 基于 `async-openai` 的 OpenAI 兼容云端后端。
- **`llama-cpp`** — 基于 `llama.cpp` 的本地推理（支持 `cuda`、`vulkan`、`metal`、`rocm` 子特性）。
- **`cuda`**、**`vulkan`**、**`metal`**、**`rocm`** — 本地引擎的 GPU 加速（仅能选择其一）。

<br>

#### 许可证

<sup>
本项目基于 <a href="LICENSE-APACHE">Apache License, Version 2.0</a> 许可。
</sup>

<br>

<sub>
除非你明确说明，否则任何有意提交到本仓库的贡献，若由你提供，则默认以与上述相同的条款进行双重许可，而不附加任何额外条款或条件。
</sub>