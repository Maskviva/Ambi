# Ambi 🦀

> [中文](README_zh.md)

Ambi is a flexible, highly customizable AI Agent framework built entirely in Rust. It features a dual-engine
architecture supporting both local inference and cloud APIs, alongside a powerful multi-tool calling system and
intelligent context management.

## ✨ Key Features

- **Dual-Engine Architecture**:
    - **Local Engine**: Powered by `llama.cpp`, supporting GGUF models with hardware acceleration for CUDA, Vulkan,
      Metal, etc.
    - **Cloud Engine**: Fully compatible with OpenAI-spec APIs (e.g., DeepSeek, SiliconFlow, Groq).
- **Parallel Multi-Tool Calling**: Capable of parsing and executing multiple `[TOOL_CALL]` instructions in a single
  response, significantly boosting complex task efficiency.
- **Fine-grained Tool Control**: Configure `timeout_secs` and `max_retries` independently for each tool.
- **Built-in Chat Templates**: Native support for Chatml, Llama3, Gemma, DeepSeek, and more, with support for fully
  custom templates.
- **Intelligent Context Eviction**: A safe eviction algorithm that identifies `User` messages as cutting points to
  prevent token overflow while maintaining logical flow.
- **Streaming Reasoning Support**: Automatically identifies and formats reasoning tags like `<think>` for models like
  DeepSeek.

## 📦 Installation

Add Ambi to your `Cargo.toml`:

```toml
[dependencies]
ambi = "0.1.0"

# For cloud-only usage (faster compilation):
# ambi = { version = "0.1.0", default-features = false, features = ["openai-api"] }
```

## 🚀 Quick Start

### 1. Instantiate an Agent

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
        .preamble("You are a helpful assistant.");

    let response = agent.chat("Hello!").await;
    println!("{}", response);

    Ok(())
}
```

### 2. Define a Custom Tool

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
        ToolDefinition::new(Self::NAME, "Add two numbers together")
            .parameter("a", "number", "First number")
            .parameter("b", "number", "Second number")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr> {
        Ok(args.a + args.b)
    }
}

// Mount when creating Agent
// let mut agent = Agent::make(config)?.tool(AddTool)?;
```

## 🛠️ Hardware Acceleration (Local)

Enable specific features for GPU acceleration:

- **CUDA**: `cargo build --features cuda`
- **Metal**: `cargo build --features metal`
- **Vulkan**: `cargo build --features vulkan`

## ⚖️ License

Licensed under the **Apache-2.0** License.