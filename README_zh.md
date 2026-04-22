# Ambi 🦀

> [English](./README.md)

Ambi 是一个基于 Rust 开发的灵活、可定制的 AI Agent (智能体) 框架。它采用双引擎设计，支持本地模型推理与云端 API
调用，并提供强大的工具调用（Tool Calling）系统和智能上下文管理能力。

## ✨ 核心特性

- **双引擎架构 (Dual-Engine)**:
    - **本地引擎**: 基于 `llama.cpp`，支持 GGUF 模型，兼容 CUDA、Vulkan、Metal 等硬件加速。
    - **云端引擎**: 完美兼容 OpenAI 规范的 API（如 DeepSeek, SiliconFlow, Groq 等）。
- **多工具并行调用 (Multi-Tool Calling)**: 能够单次解析并执行模型输出中的多个工具调用指令，大幅提升复杂任务的处理效率。
- **精细化工具控制**: 支持为每个工具独立配置超时时间 (`timeout_secs`) 和最大重试次数 (`max_retries`)。
- **内置对话模板**: 原生支持 Chatml, Llama3, Gemma, Deepseek 等主流模型模板，并支持完全自定义模板。
- **智能记忆截断**: 采用安全的上下文剔除算法，自动寻找对话中的 User 消息作为切割点，防止 Token 溢出的同时确保逻辑连贯。
- **流式思维链处理**: 自动识别并格式化类似 DeepSeek 的 `<think>` 标签，让 Agent 的推理过程优雅展示。

## 📦 安装

在你的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
ambi = "0.1.3"

# 如果你只需要云端 API 支持，可以禁用默认的本地引擎以加快编译
# ambi = { version = "0.1.3", default-features = false, features = ["openai-api"] }
```

## 🚀 快速开始

### 1. 实例化 Agent

```rust
use ambi::{Agent, EngineConfig, OpenAIEngineConfig};
use ambi::llm::chat_template::ChatTemplateType;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = EngineConfig::OpenAI(OpenAIEngineConfig {
        api_key: "your-api-key".into(),
        base_url: "[https://api.deepseek.com](https://api.deepseek.com)".into(),
        model_name: "deepseek-chat".into(),
        temp: 0.7,
        top_p: 0.9,
    });

    let mut agent = Agent::make(config)?
        .template(ChatTemplateType::Deepseek)
        .preamble("你是一个专业的助手。");

    let response = agent.chat("你好！").await;
    println!("{}", response);

    Ok(())
}
```

### 2. 定义并使用工具

```rust
use ambi::agent::tool::{Tool, ToolDefinition, ToolErr};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
struct AddArgs {
    a: f64,
    b: f64
}

struct AddTool;

#[async_trait]
impl Tool for AddTool {
    const NAME: &'static str = "add";
    type Args = AddArgs;
    type Output = f64;

    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(Self::NAME, "计算两个数字之和")
            .parameter("a", "number", "第一个加数")
            .parameter("b", "number", "第二个加数")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr> {
        Ok(args.a + args.b)
    }
}

// 在创建 Agent 时挂载
// let mut agent = Agent::make(config)?.tool(AddTool)?;
```

## 🛠️ 硬件加速 (本地模型)

启用对应的 Feature 以获得 GPU 加速：

- **CUDA**: `cargo build --features cuda`
- **Metal**: `cargo build --features metal`
- **Vulkan**: `cargo build --features vulkan`

## ⚖️ 开源协议

本项目采用 **Apache-2.0** 协议开源。