# 快速开始（Rust，0.4.1）

这份指南从最小的文本 Agent 开始。Ambi 是一个 **Agent Runtime**，不是强制性的多智能体框架：选择 Engine，把可共享能力装进 `Agent`，为会话创建独立的 `AgentState`，最后使用某个 `Pipeline` 执行。

## 1. 添加依赖

```toml
[dependencies]
ambi = "0.4.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

默认开启 `openai-api` Feature。如果你使用的 Registry 尚未发布 `0.4.1`，可改用匹配源码的 `path` 或 Git 依赖。先设置环境变量 `OPENAI_API_KEY`。

## 2. 完成第一次聊天

编写 `src/main.rs`：

```rust
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::var("OPENAI_API_KEY")?;
    let cfg = OpenAIEngineConfig::create(key, "gpt-4o-mini")
        .temp(0.7);

    let agent = Agent::make(LLMEngineConfig::OpenAI(cfg)).await?
        .preamble("你是一个简洁的助手。");
    let state = AgentState::new_shared("session-1");
    let runner = ChatRunner::default();

    let answer = runner.chat(&agent, &state, "你好，Ambi！").await?;
    println!("{answer}");
    Ok(())
}
```

执行 `cargo run`。`Agent::make` 初始化引擎；`ChatRunner::default()` 使用 ReAct 风格的聊天／工具循环，默认单次循环内最大工具并发为 5。

## 3. 切换模型后端

**OpenAI 兼容端点：**

```rust
let cfg = OpenAIEngineConfig::create(api_key, "your-model")
    .base_url("https://your-endpoint.example/v1".into())
    .temp(0.2);
```

**本地 GGUF / llama.cpp**（仅原生平台）：

```toml
ambi = { version = "0.4.1", default-features = false, features = ["llama-cpp"] }
```

```rust
use ambi::llm::providers::llama_cpp::config::LlamaEngineConfig;
let cfg = LlamaEngineConfig::builder("./models/model.gguf")
    .n_ctx(4096)
    .n_tokens(512)
    .build();
let engine_cfg = LLMEngineConfig::Llama(cfg);
```

GPU 可选择 `cuda`、`vulkan`、`metal`、`rocm` 其中合适的一种，不要同时启用互斥的硬件后端。浏览器 WebAssembly 走 OpenAI 兼容后端，而非 llama.cpp。

## 4. 继续学习

- [设计理念](/zh/advanced/design-philosophy)：为什么要区分 Agent、Pipeline 和 Leaf
- [基础 Agent](/zh/guide/basic-agent)：独立会话与动态上下文
- [工具](/zh/guide/tools)：向模型提供类型化 Rust 操作
- [Leaf](/zh/guide/leaf)：通过 `sub-agent` 可选功能实现一次性任务委派
- [自定义 Pipeline](/zh/advanced/custom-pipeline)：自行定义智能体的执行方式

对应源码示例：[`examples/chat_cloud.rs`](https://github.com/Maskviva/Ambi/blob/main/examples/chat_cloud.rs)、[`examples/chat_local.rs`](https://github.com/Maskviva/Ambi/blob/main/examples/chat_local.rs)。
