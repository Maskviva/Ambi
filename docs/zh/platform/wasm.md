# WebAssembly：API 尽量一致，平台语义各有边界

Ambi 提供适配浏览器的 WASM 异步机制，可使用 OpenAI 兼容 API 或自定义 Engine。`llama-cpp` 在 `wasm32` 目标上会被**编译期拒绝**。

## 基础依赖

```toml
[dependencies]
ambi = { version = "0.4.1", default-features = false, features = ["openai-api"] }
tokio = { version = "1", features = ["sync", "macros"] }
```

使用常规 WASM 工具链编译到 `wasm32-unknown-unknown`，并根据应用增加 UI 和构建依赖。仓库中有完整应用级示例：[`examples/webAssembly`](https://github.com/Maskviva/Ambi/tree/main/examples/webAssembly)。

## 与原生平台的差异

| 能力 | Native | 浏览器 WASM |
| --- | --- | --- |
| OpenAI 兼容 HTTP | 支持 | 浏览器 fetch API |
| 流式输出 | 支持 | ReadableStream 适配 |
| 本地 llama.cpp | 支持 | **不支持** |
| 原生 GPU 后端 | 按 Feature | **不支持** |
| 自定义 Engine | 支持 | 受 WASM 执行环境约束 |
| 异步执行 | Tokio | WASM 本地适配 |

浏览器仍受到自身的网络和安全模型约束：CORS 与**明文暴露 API 凭据**必须由应用负责。不要把有权限的云端密钥放进公开的前端 Bundle。

API 共享不代表两个平台拥有完全相同的资源与线程语义。参见 [架构](/zh/advanced/architecture) 和 [配置](/zh/guide/configuration)。
