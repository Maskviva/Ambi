# 自定义 Engine：替换推理，而非替换 Runtime

Ambi 将**如何调用模型**与**如何执行一次任务**分离。可以实现 `LLMEngineTrait` 接入私有协议、构建确定性测试 Mock，或对接不兼容 OpenAI API 的本地推理服务。

## 0.4.1 接口

```rust
use ambi::{impl_as_any, LLMEngineConfig};
use ambi::llm::LLMEngineTrait;
use ambi::types::LLMRequest;
use ambi::error::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

struct EchoEngine;

#[async_trait]
impl LLMEngineTrait for EchoEngine {
    impl_as_any!();

    async fn chat(&self, _req: LLMRequest) -> Result<String> {
        Ok("Echo from custom engine".into())
    }

    async fn chat_stream(&self, _req: LLMRequest, tx: Sender<Result<String>>) {
        let _ = tx.send(Ok("Echo from custom engine".into())).await;
    }

    fn reset_context(&self) { /* 无状态后端 */ }
}

let cfg = LLMEngineConfig::Custom(Box::new(EchoEngine));
```

原生项目需要 `ambi`、`async-trait` 和 Tokio。接着调用 `Agent::make(cfg).await?`。`supports_multimodal()` 默认返回 `false`，只有模型真正支持图片时才应覆盖。`as_any()` 是必需方法，`impl_as_any!()` 用于生成实现。

当前主要返回协议是 **`String` 文本**，不是完整的结构化模型事件。工具调用文本必须与可替换的 `ToolCallParser` 匹配。

## 自定义 Pipeline 如何取得 Engine

`Agent` 的内部字段不是可直接修改的公开字段。使用：

```rust
let engine = agent.llm_engine(); // Arc<LLMEngine>
```

若需要自定义 Tokenizer，可在 `LLMEngine::load(...)` 后调用 `.with_custom_tokenizer(...)`，再通过 `Agent::from_engine(Arc::new(engine))` 构建 Agent。Tokenizer 主要为上下文驱逐提供**近似预算统计**，不能天然代表模型精确 Token 数。

## Runtime 边界

- OpenAI 兼容后端有内部并发上限和请求超时。
- 本地 llama.cpp 具有 Worker 队列和按会话判断缓存归属的机制。
- `reset_context()` 操作 Engine；共享本地 Engine 时并不只作用于当前 Session。
- 流式错误通过通道传递 `Err`，自定义后端应认真处理终止与取消。

`Agent::with_custom_engine` 和 `LLMEngine::from_custom` 已弃用，推荐 `LLMEngineConfig::Custom`。

参见 [自定义 Pipeline](/zh/advanced/custom-pipeline) · [架构](/zh/advanced/architecture)。
