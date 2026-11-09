# 原生平台：Linux、Windows、macOS

Ambi 原生端支持 OpenAI 兼容 API、本地 llama.cpp 和自定义 Engine。尽可能让平台差异停留在 Engine 层，不要让上层 Pipeline 被推理硬件绑定。

## Runtime 与构建

```toml
[dependencies]
ambi = "0.4.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

建议使用 Tokio 多线程 Runtime，因为 `Agent::make()` 通过内部阻塞任务适配器进行引擎初始化。不要在未针对具体后端测试的情况下，假设 `current_thread` 一定满足所有初始化要求。

本地推理：

```toml
ambi = { version = "0.4.1", default-features = false, features = ["llama-cpp"] }
```

GPU Feature 包括 `cuda`、`vulkan`、`metal`、`rocm`，**不能同时启用多个**。所需 SDK、驱动和构建工具取决于操作系统；合适的平台也可使用 `dynamic-link`。

可以从 [`LlamaEngineConfig::builder()`](/zh/guide/configuration) 的保守默认值开始，按硬件和模型调整上下文、Batch、模型路径及 GPU 选项。本地推理使用 Worker 队列，不同会话交替执行可能重新 Prefill KV Cache。

## 基础验证

```bash
cargo check --no-default-features --features openai-api
cargo test --no-default-features --features openai-api
cargo test --no-default-features --features "openai-api,sub-agent" --test leaf_subagent_test
```

最后一条用于测试可选 Leaf，需要兼容的环境；它不会自动包含在所有默认测试命令中。

继续：[快速开始](/zh/guide/getting-started) · [架构](/zh/advanced/architecture)。
