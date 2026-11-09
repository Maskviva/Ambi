# 多模态输入：能力由 Engine 决定

Ambi 的请求格式支持混合文本和图片。能否实际理解图片取决于**所选 Engine、具体模型以及编译 Feature**。

## 发送图片

```rust
use ambi::agent::pipeline::Pipeline;
use ambi::ContentPart;

let parts = vec![
    ContentPart::Text { text: "描述这张图片".into() },
    ContentPart::Image { base64: image_data },
];
let answer = runner.execute(&agent, &state, parts).await?;
```

`image_data` 需要由应用提供。`base64` 字段可以传 Base64 或 Data URL；某些后端也接受 HTTP 图片 URL。但不能假设所有后端都支持完全相同的图片形式。

## OpenAI 兼容 API

`OpenAIEngineConfig::create()` 会根据已知模型名前缀设置 `supports_vision`。如果兼容端点使用未知模型名但实际支持图片，需要显式 `.with_vision(true)`。这个标记是**能力声明**，不会主动探测远程模型。若 Engine 报告 `supports_multimodal() == false`，框架会提前拒绝图片输入。

## 本地 llama.cpp

视觉处理需要支持的模型、`llama-cpp` 后端和 `mtmd` Feature。有些架构还需要匹配的外部 `mmproj` 文件，另一些采用集成视觉能力。开启 Feature 并不能将纯文本 GGUF 变成视觉模型。

```toml
ambi = { version = "0.4.1", default-features = false, features = ["llama-cpp", "mtmd"] }
```

不同 Provider 对图片 Token 的估算与处理可能不同，不能把纯文本驱逐阈值看成精确的多模态预算。参见 [配置](/zh/guide/configuration) 与 [上下文驱逐](/zh/advanced/context-eviction)。
