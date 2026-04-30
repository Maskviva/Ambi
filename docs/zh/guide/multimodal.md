# 多模态输入

如果引擎后端支持图像输入，Ambi 就可以处理图片。这适用于 OpenAI 的视觉模型（gpt-4o、gpt-4-vision）和 llama.cpp 的 VLM 模型。

## 发送图片

用 `ContentPart::Image` 加上 base64 编码的图片字符串：

```rust
use ambi::ContentPart;
use ambi::types::Message;

let parts = vec![
    ContentPart::Text { text: "这张图里有什么？".into() },
    ContentPart::Image { base64: image_base64_string },
];

let reply = runner.execute(&agent, &state, parts).await?;
```

也可以用便捷方法：

```rust
let msg = Message::user_multimodal("描述这张图", &image_base64);
```

### URL 支持

`ContentPart::Image` 的 `base64` 字段可以传 base64 数据字符串，也可以传 HTTP URL。用 OpenAI 后端时直接传 URL 更高效：

```rust
ContentPart::Image { base64: "https://example.com/photo.jpg".into() }
```

## 快速失败

如果你给不支持多模态的引擎发了图片，会立即收到 `EngineError`：

```
Security Check Failed: The current LLM engine does not support multimodal (image) inputs.
```

这个检查在发送任何 token 之前就跑完，不会浪费 API 调用。

## 引擎支持情况

| 引擎 | 多模态 | 备注 |
|------|--------|------|
| OpenAI（gpt-4o、gpt-4-vision） | 支持 | URL 或 base64 |
| Llama.cpp 视觉模型 | 支持（需 `mtmd` 特性） | Qwen2-VL、LLaVA |
| 自定义引擎 | 取决于 `supports_multimodal()` | trait 方法默认返回 false |
