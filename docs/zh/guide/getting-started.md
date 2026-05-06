# 快速开始

这篇文章带你 5 分钟跑通一个可工作的 Agent。你需要：

- Rust 1.75+
- 一个 API Key（用云后端）或一个 GGUF 模型文件（本地推理）

## 1. 添加依赖

```toml
[dependencies]
ambi = "0.3"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Ambi 默认启用 `openai-api` 特性。如果你只用云后端，编译很快。

本地推理（llama.cpp）：

```toml
ambi = { version = "0.3", default-features = false, features = ["llama-cpp"] }
```

GPU 加速子特性：`cuda`、`vulkan`、`metal`、`rocm`——只能选一个。

```toml
ambi = { version = "0.3", features = ["llama-cpp", "cuda"] }
```

## 2. 最小 Agent

```rust
use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LLMEngineConfig::OpenAI(ambi::OpenAIEngineConfig {
        api_key: std::env::var("OPENAI_API_KEY")?,
        base_url: "https://api.openai.com/v1".into(),
        model_name: "gpt-4o".into(),
        temp: 0.7,
        top_p: 0.95,
    });

    let agent = Agent::make(config).await?
        .preamble("你是一个乐于助人的助手。")
        .template(ambi::ChatTemplateType::Chatml);

    let state = AgentState::new_shared("session-001");
    let runner = ChatRunner::default();

    let reply = runner.chat(&agent, &state, "你好！").await?;
    println!("{}", reply);

    Ok(())
}
```

`AgentState::new_shared("...")` 是一个便捷构造器，内部用 `Arc<RwLock<>>` 包装状态以实现线程安全。`session_id` 参数为分布式追踪和 KV Cache 槽位分配建立物理唯一性。`Agent::make` 加载引擎（llama.cpp 时在阻塞线程上加载），然后用 builder 链式配置。`ChatRunner::default()` 创建默认最大并发为 5 的运行器。

## 3. 切换引擎

云和本地之间切换只需要改一行——换配置枚举变体：

```rust
// 云
let config = LLMEngineConfig::OpenAI(openai_cfg);

// 本地（需要 "llama-cpp" 特性）
let config = LLMEngineConfig::Llama(llama_cfg);
```

其他一切——工具、模板、流、格式化器——都不变。

## 4. 运行时要求

Ambi 需要 Tokio 的 `rt-multi-thread` 特性。单线程运行时跑不起来，因为 `Agent::make` 内部会调用 `spawn_blocking`。

## 下一步

- [基础 Agent](/zh/guide/basic-agent) —— 系统提示词、聊天模板、多轮对话
- [工具调用](/zh/guide/tools) —— 让 Agent 能调用 Rust 函数
- [配置详解](/zh/guide/configuration) —— 驱逐策略、迭代限制等
