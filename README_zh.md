# Ambi

<p align="center">
  <img src="docs/public/logo.svg" alt="Ambi 标志" width="96" />
</p>

<p align="center">
  <strong>小而可组合的 Rust AI Agent Runtime。</strong><br />
  框架提供运行机制，智能体由你定义。
</p>

<p align="center">
  <a href="https://crates.io/crates/ambi"><img alt="crates.io" src="https://img.shields.io/crates/v/ambi" /></a>
  <a href="https://docs.rs/ambi"><img alt="docs.rs" src="https://img.shields.io/docsrs/ambi" /></a>
  <a href="LICENSE"><img alt="Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue" /></a>
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.zh.md">简体中文</a> ·
  <a href="https://maskviva.github.io/Ambi/zh/">在线文档</a> ·
  <a href="CHANGELOG_zh.md">更新日志</a> ·
  <a href="examples/">示例</a>
</p>

---

**Ambi** 是一个基于 Rust 的轻量级 AI 智能体运行时（Agent Runtime）。它提供 LLM
推理、工具执行、上下文管理与执行循环所需的基础机制，但不规定所有智能体必须如何思考、规划或协作。

你可以直接使用内置的 `ChatRunner` 构建 ReAct 风格的工具调用智能体，也可以实现自己的 `Pipeline`。模型可以来自 OpenAI 兼容
API、本地 `llama.cpp`，或你自行实现的 Engine。需要什么能力就组合什么能力，不需要的部分无需引入。

> **核心理念：组合优于继承。** Ambi 提供执行机制，而不是一套强制性的认知架构。内置功能决定使用的起点；开发者通过 Trait
> 可以探索的能力不应被这些用例限制。

## 为什么选择 Ambi？

- **小核心，开放扩展点。** `LLMEngineTrait`、`Pipeline`、工具解析器、流格式化器和 Tokenizer 等可以替换，不需要继承一个庞大的
  Agent 基类。
- **Agent 与 State 分离。** `Agent` 持有可以共享的模型、工具和配置；`AgentState` 管理会话历史与动态上下文，同一个 Agent
  可以用于不同会话。
- **工具是一等能力。** 使用 Rust 异步工具、JSON Schema 参数定义、并行调用、超时限制与区分幂等性的重试策略。
- **云端与本地统一。** 支持 OpenAI 兼容 Chat Completions API、本地 GGUF 模型推理，以及自定义 Engine。
- **显式的上下文管理。** 提供 Token 统计、FIFO 历史裁剪、动态上下文和可选的裁剪回调，不强制绑定记忆数据库。
- **按平台选择能力。** 支持原生 Rust 和浏览器 WebAssembly；`llama-cpp` 后端仅支持原生目标平台。
- **Leaf 临时任务委派（可选）。** 在现有 Pipeline 中按需创建一次性任务执行单元，无需引入完整的子 Agent 层级体系。

## 设计模型：树、枝与叶

```text
                      Agent
               （模型、工具、配置）
                        │
             ┌──────────┴──────────┐
             │                     │
         ChatRunner             自定义 Pipeline
        （ReAct 执行）           （实现 Trait）
             │
       可选的 spawn_leaf
             │
      ┌──────┴──────┐
      │             │
    Leaf A        Leaf B           → 只返回最终结果
 （独立状态）    （独立状态）

  AgentState：独立于 Agent、交给 Pipeline 使用的会话状态
```

这里的树是**设计比喻**，不是要求代码形成固定的继承层级。

**Agent 是树，Pipeline 是枝，Leaf 是叶。** 树定义可复用的能力，枝决定如何执行，叶则为一次明确的任务而生，完成后即被丢弃。不同的枝可以发展出不同的能力；其他开发者也可以通过
Trait 嫁接自己的实现。

这正是 Ambi 想保持的边界：**机制尽量稳定，策略留给开发者。**

| 核心组件             | 职责                    |
|------------------|-----------------------|
| `Agent`          | 共享 Engine、系统提示词、工具与配置 |
| `AgentState`     | 保存会话历史、动态上下文与扩展状态     |
| `Pipeline`       | 定义一次请求如何被执行           |
| `ChatRunner`     | 内置的 ReAct 聊天与工具执行循环   |
| `LLMEngineTrait` | 为不同模型后端提供统一推理接口       |
| `Tool`           | 把类型化外部操作暴露给模型         |
| `Leaf`（可选）       | 执行一次临时委派任务，并返回最终结果    |

## 快速开始

### 1. 添加依赖

如果使用 OpenAI 兼容 API，默认 Feature 即可：

```toml
[dependencies]
ambi = "0.4.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

如果 `0.4.1` 尚未发布到你使用的 Registry，可改用本地 `path` 或 Git 依赖。以下代码以**仓库中的 0.4.1 源码 API** 为准。

### 2. 创建智能体

先设置环境变量 `OPENAI_API_KEY`，然后编写 `src/main.rs`：

```rust
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let engine = OpenAIEngineConfig::create(api_key, "gpt-4o-mini")
        .temp(0.7);

    let agent = Agent::make(LLMEngineConfig::OpenAI(engine))
        .await?
        .preamble("You are a helpful assistant.");

    let state = AgentState::new_shared("conversation-1");
    let runner = ChatRunner::default();

    let answer = runner.chat(&agent, &state, "你好，Ambi！").await?;
    println!("{answer}");
    Ok(())
}
```

`ChatRunner::default()` 不会自动启用 Leaf，即使编译时已经打开了 `sub-agent` Feature 也是如此。

可以从仓库示例继续学习：[`chat_cloud`](examples/chat_cloud.rs)、[`chat_stream`](examples/chat_stream.rs)、[
`chat_local`](examples/chat_local.rs)、[`custom_tool`](examples/custom_tool.rs)。

## Leaf：任务委派，不是另一套 Agent 体系

Leaf 的存在是为了**完成一次被委派的任务**。父级模型把它看作名为 `spawn_leaf` 的工具，但工具内部执行的是一个临时的聊天循环，而不是创建一个拥有长期身份的完整
`Agent`。

一次 Leaf 调用遵循以下规则：

1. 创建全新的 `AgentState`，不继承父级的对话历史。
2. 共享父级的 Engine，并使用父级原有工具与聊天配置的快照；部分参数可以通过 `LeafConfig` 覆盖。
3. Leaf 的工具快照不包含 `spawn_leaf`，因此从结构上禁止继续递归生成 Leaf。
4. **只把最终回答返回给父级**，不把中间轮次的工具展示块和推理文本复制到父级的回答上下文。
5. 任务结束后丢弃临时会话状态，不建立新的持久化 Agent 实体。

例如：生成文档摘要、快速理解某个项目模块，或者让多个独立任务分别产出结论。这些任务需要自己的工作上下文，但不应该把全部过程塞进父级历史。

**第一步：显式开启 Cargo Feature。**

```toml
ambi = { version = "0.4.1", features = ["sub-agent"] }
```

**第二步：在需要的 ChatRunner 上启用。**

```rust
use ambi::{ChatRunner, LeafConfig};

// 简单开启：ChatRunner::default().with_leaf()
let runner = ChatRunner::default ().with_leaf_config(
LeafConfig::new()
.with_max_iterations(6)
.with_timeout_secs(120),
);

// agent 和 state 为已经创建好的实例：
let answer = runner.chat(
& agent,
& state,
"阅读架构文档，并将独立的摘要任务委派给 Leaf。",
).await?;
```

是否调用 `spawn_leaf` 由模型决定；`with_leaf()` 只是让该能力可用，并不会无条件创建 Leaf。如果模型在同一轮提出多个独立的
Leaf 工具调用，运行时可以并行处理，但实际吞吐仍受模型后端和工具资源限制。

> **必须明确的边界：** Leaf 隔离的是**会话历史**，不是计算资源或外部副作用。它共享父级 Engine，默认继承**全部父级工具**
> ，包括具有副作用的工具。本地 `llama.cpp` 只有一个推理 Worker / KV Cache，父级与 Leaf 会话切换可能造成重复 Prefill。Leaf
> 不是沙箱，也不是长期 Worker。当前内置版本尚无工具 Allow-list 或独立模型选择机制。

当前实现位于 [`src/agent/pipeline/chat_runner/leaf.rs`](src/agent/pipeline/chat_runner/leaf.rs)。

## 给智能体添加工具

实现 `Tool` Trait 即可把异步 Rust 操作开放给模型。每个工具显式提供 JSON Schema、超时、重试次数，以及是否具有幂等性。

```rust
use ambi::types::{Tool, ToolDefinition, ToolErr};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct AddArgs {
    a: i64,
    b: i64
}

#[derive(Serialize)]
struct AddResult {
    sum: i64
}

struct AddNumbers;

#[async_trait]
impl Tool for AddNumbers {
    const NAME: &'static str = "add_numbers";
    type Args = AddArgs;
    type Output = AddResult;

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "计算两个整数之和".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": { "type": "integer" },
                    "b": { "type": "integer" }
                },
                "required": ["a", "b"]
            }),
            timeout_secs: Some(5),
            max_retries: Some(1),
            is_idempotent: true,
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr> {
        Ok(AddResult { sum: args.a + args.b })
    }
}

// 构建 Agent 时：
// let agent = Agent::make(config).await?.tool(AddNumbers)?;
```

以上手写工具示例还需要在应用中添加 `async-trait`、`serde`（开启 `derive`）和 `serde_json` 依赖。若希望减少样板代码，可以开启
`macro` Feature，参考 [`examples/custom_tool_in_macro.rs`](examples/custom_tool_in_macro.rs)。宏可以生成工具参数
Schema，而上面的手动实现需要自行提供 Schema。

**带副作用的工具需要谨慎。** 不可安全重试的操作应设置 `is_idempotent: false`。超时或取消 Future，并不意味着已经提交给外部服务的操作一定可以撤销。

## 流式输出与上下文管理

`ChatRunner` 同时提供流式接口（使用 `StreamExt` 时需添加 `futures` 依赖）：

```rust
use futures::StreamExt;

let mut stream = runner.chat_stream( & agent, & state, "介绍一下这个项目").await?;
while let Some(chunk) = stream.next().await {
print!("{}", chunk?);
}
```

`AgentState` 独立于 `Agent`，可以在多轮会话之间持续使用。临时背景信息可以放入动态上下文，而不是写入固定的 System Prompt：

```rust
state.write().await.set_dynamic_context("当前相关的项目文档：...");
// 不再需要时，主动清理：
state.write().await.clear_dynamic_context();
```

当会话历史超过配置的 Token 预算时，Ambi 会根据统计值按 FIFO 裁剪较早的消息；系统提示词不参与这条历史队列。需要注意：这**不等于
**对所有 Provider 精确计算 Token，也不代表按语义完整轮次裁剪，更不等于开箱即用的长期持久化记忆。应用可以通过 `on_evict`
回调接入自己的存储策略。

## 模型后端、Features 与生态

### Cargo Features

| Feature                              | 默认开启 | 用途                                    |
|--------------------------------------|------|---------------------------------------|
| `openai-api`                         | 是    | OpenAI 兼容 API 后端                      |
| `llama-cpp`                          | 否    | 通过 `llama.cpp` 在本地运行 GGUF 模型          |
| `cuda` / `vulkan` / `metal` / `rocm` | 否    | 原生 GPU 加速，**只能选择一种**；自动开启 `llama-cpp` |
| `mtmd`                               | 否    | `llama.cpp` 多模态能力                     |
| `dynamic-link`                       | 否    | 本地后端的动态链接选项                           |
| `macro`                              | 否    | 工具 / Agent 声明过程宏                      |
| `sub-agent`                          | 否    | 编译 Leaf 功能；还需调用 `with_leaf()` 显式启用    |

如果只需要本地推理，不需要云端 API：

```toml
ambi = { version = "0.4.1", default-features = false, features = ["llama-cpp"] }
```

```rust
use ambi::llm::providers::llama_cpp::config::LlamaEngineConfig;
use ambi::LLMEngineConfig;

let local = LlamaEngineConfig::builder("./models/model.gguf")
.n_ctx(4096)
.n_tokens(512)
.max_tokens(512)
.build();
let config = LLMEngineConfig::Llama(local);
```

GPU 后端需要对应的系统工具链与库。`llama-cpp` 无法编译到 `wasm32`，浏览器端可使用 `openai-api` 或自定义 Engine。浏览器 API
请求还受到服务端 CORS 策略限制；不要在公开客户端暴露高权限 API Key。

### 可选 Crate 与语言绑定

Ambi 不把所有高级功能都塞进核心 Crate：

| 包 / 目录                                          | 作用                                                  |
|-------------------------------------------------|-----------------------------------------------------|
| [`ambi-pipelines`](ambi-pipelines/)             | RAG、CoT Self-Consistency、Tree-of-Thoughts、Reflexion |
| [`ambi-memory`](ambi-memory/)                   | KV、Summary、Semantic Memory 接口，以及内存版 KV / Summary 实现 |
| [`ambi-macros`](ambi-macros/)                   | Agent / Tool 过程宏                                    |
| [`bindings/python`](bindings/python/)           | Python 绑定（包名 `ambi-python`）                         |
| [`bindings/node`](bindings/node/)               | Node.js 绑定（包名 `@maskviva/ambi-node`）                |
| [`examples/webAssembly`](examples/webAssembly/) | 浏览器 / WASM 示例                                       |

扩展 Crate 和其他语言绑定有各自的发布版本与能力覆盖范围，使用前请查看对应目录。**提供语义记忆 Trait 不代表内置向量数据库**
，内存 Provider 也不等于持久化存储。

## 文档与开发

- [快速开始](docs/zh/guide/getting-started.md) · [自定义工具](docs/zh/guide/tools.md) · [自定义 Pipeline](docs/zh/advanced/custom-pipeline.md)
- [架构设计](docs/zh/advanced/architecture.md) · [设计理念](docs/zh/advanced/design-philosophy.md) · [WASM](docs/zh/platform/wasm.md)
- [示例代码](examples/) · [更新日志](CHANGELOG_zh.md) · [English Changelog](CHANGELOG.md)

```bash
cargo fmt --all -- --check
cargo clippy --no-default-features --features openai-api -- -D warnings
cargo test --no-default-features --features openai-api
cargo test --no-default-features --features "openai-api,sub-agent" --test leaf_subagent_test
```

Leaf 测试需要显式启用 `sub-agent` Feature。依赖在线模型的测试还需要配置对应服务；编译或单元测试通过并不能替代性能测试与真实生产验证。

## 许可证

采用 [Apache License 2.0](LICENSE) 授权。欢迎参与贡献：尽量保持核心小而清晰、扩展边界明确、执行行为可预期。
