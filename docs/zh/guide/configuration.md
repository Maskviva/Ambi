# 配置与 Feature 边界

Ambi 的配置对应其设计分工：分别选择模型推理**机制**、执行**策略**和会话**状态**。

## Cargo Features（0.4.1）

| Feature | 作用 | 默认 |
| --- | --- | --- |
| `openai-api` | OpenAI 兼容 API | **开启** |
| `llama-cpp` | 本地 GGUF（仅原生平台） | 关闭 |
| `mtmd` | llama.cpp 多模态支持 | 关闭 |
| `cuda`、`vulkan`、`metal`、`rocm` | 硬件加速；按平台选择合适的一种 | 关闭 |
| `dynamic-link` | llama.cpp 动态链接 | 关闭 |
| `macro` | 重新导出 `ambi-macros` | 关闭 |
| `sub-agent` | 编译 ChatRunner Leaf 能力 | 关闭 |

编译 `sub-agent` **不会**自动启用 Leaf；仍需显式调用 `ChatRunner::with_leaf()`。

## Agent 配置

`Agent::make(cfg)` 初始化共享推理资源，后续 Builder 调整配置：

```rust
use ambi::config::EvictionStrategy;

let agent = Agent::make(engine_config).await?
    .preamble("你是一个可靠的助手。")
    .max_iterations(8)
    .with_eviction_strategy(EvictionStrategy { max_safe_tokens: 4096 })
    .with_standard_formatting();
```

`AgentConfig` 默认值：空系统提示词、ChatML 模板、10 次迭代、8,000 的近似上下文安全 Token 阈值。`ChatRunner::default()` 每个执行循环内工具并发上限为 5。**它们不是进程全局预算。**

## OpenAI 兼容配置

```rust
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;

let cfg = OpenAIEngineConfig::create(api_key, "your-model")
    .base_url("https://your-endpoint.example/v1".into())
    .temp(0.1)
    .top_p(1.0)
    .with_vision(false);
```

`create()` 默认使用 OpenAI URL、温度 `0.0`、top-p `1.0`；`supports_vision` 按识别出的模型名前缀推断。若是支持图片的未知兼容端点，可显式 `.with_vision(true)`。这表示调用方**声明**模型能力，并非实时探测。

OpenAI Engine 另外具有内部请求并发上限（默认 32）与请求超时（默认 300 秒），它们与 ChatRunner 的工具并发不是一回事。

## llama.cpp 配置

```rust
use ambi::llm::providers::llama_cpp::config::LlamaEngineConfig;
let cfg = LlamaEngineConfig::builder("./models/model.gguf")
    .n_ctx(4096)
    .n_tokens(512)
    .max_tokens(1024)
    .build();
```

Builder 从保守的 CPU 配置起步。模型文件必须存在。校验会拒绝 `n_ctx = 0`、超出 `1..=n_ctx` 的 Batch 大小和非正数 `max_tokens`。GPU 功能需要兼容构建。详见 [原生平台](/zh/platform/native)。

## Leaf 配置

开启 `sub-agent` 后，`LeafConfig` 可覆盖工具名、描述、提示词、迭代上限、内部工具并发和超时；它不提供独立 Engine Registry 或内置工具 Allow-list。参见 [Leaf](/zh/guide/leaf)。

## 错误与预算的真实语义

上下文驱逐预算**不等于**精确保证永不超窗；模型模板、动态上下文、图片和不同 Provider 的 Token 统计可能不同。工具超时和禁用非幂等重试，也不意味着外部副作用可以自动回滚。需要更强保证的应用应自己建立相应机制。
