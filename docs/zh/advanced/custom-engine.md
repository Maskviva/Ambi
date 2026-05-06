# 自定义引擎

如果内置的 OpenAI 和 llama.cpp 引擎不够用，你可以用自己的后端。

## 什么时候需要写自定义引擎

- 想用私有模型走自定义协议
- 需要一个 mock 引擎来测试（避免真实 API 调用）
- 跑了一个非 OpenAI 兼容的本地服务

## 实现 LLMEngineTrait

```rust
use ambi::llm::LLMEngineTrait;
use ambi::types::LLMRequest;
use ambi::error::Result;
use tokio::sync::mpsc::Sender;
use async_trait::async_trait;

struct MockEngine {
    reply: String,
}

#[async_trait]
impl LLMEngineTrait for MockEngine {
    async fn chat(&self, _request: LLMRequest) -> Result<String> {
        Ok(self.reply.clone())
    }

    async fn chat_stream(&self, request: LLMRequest, tx: Sender<Result<String>>) {
        let _ = tx.send(Ok("你好，".into())).await;
        let _ = tx.send(Ok("世界！".into())).await;
    }

    fn reset_context(&self) {
        // mock 不需要做什么
    }
}
```

### 必须实现的方法

| 方法 | 用途 |
|------|------|
| `chat()` | 完整响应。返回完整的输出字符串。 |
| `chat_stream()` | 流式响应。通过 mpsc `Sender` 逐块发送。 |
| `reset_context()` | 清理内部状态/KV Cache。`ChatRunner::clear_history()` 会调用。 |

### 可选方法

| 方法 | 默认行为 | 什么情况要覆盖 |
|------|---------|---------------|
| `supports_multimodal()` | 返回 `false` | 你的引擎支持图片输入 |
| `evaluate_sentence_entropy()` | 返回 `EngineError` | 你的引擎能计算 token 级别的不确定性 |

## 使用自定义引擎

通过 `LLMEngineConfig::Custom` 变体传入你的引擎：

```rust
use ambi::{Agent, LLMEngineConfig};

let agent = Agent::make(
    LLMEngineConfig::Custom(Box::new(MockEngine {
        reply: "你好，我是 Mock。".into(),
    }))
).await?;
```

这是 **v0.3.3 起推荐** 的方式。旧的 `Agent::with_custom_engine()` 方法已废弃。

注意 `LLMEngineConfig::Custom` 是同步的 —— 不需要 `spawn_blocking`，因为没有模型文件要加载。这也意味着它在 `current_thread` Tokio 运行时下也能工作。

## 用自定义引擎做测试

Mock 引擎在测试工具逻辑时很有用：

```rust
#[tokio::test]
async fn test_tool_calls() {
    let agent = Agent::make(LLMEngineConfig::Custom(Box::new(MockEngine {
        reply: "查天气[TOOL_CALL]{\"name\":\"get_weather\",\"args\":{\"city\":\"东京\"}}[/TOOL_CALL]".into(),
    }))).await?;
    // ... 测试你的工具
}
```

## 自定义分词器

默认用 `cl100k_base`（tiktoken）。如果你的模型用不同的分词器，可以换：

```rust
use ambi::llm::{LLMEngine, LLMEngineConfig};
use ambi::llm::tokenizer::TokenizerTrait;

struct MyTokenizer;

impl TokenizerTrait for MyTokenizer {
    fn count_tokens(&self, text: &str) -> Result<usize> {
        Ok(text.len()) // 粗略估算
    }
}

// 创建引擎之后：
let engine = LLMEngine::load(LLMEngineConfig::Custom(Box::new(my_engine)))?;
let engine = engine.with_custom_tokenizer(MyTokenizer);
```

> **注意：** 旧的 `LLMEngine::from_custom()` 方法自 v0.3.3 起已废弃。
> 请使用 `LLMEngine::load(LLMEngineConfig::Custom(backend))` 替代。

这会影响上下文驱逐的准确性。不准的分词器可能导致过早或过晚驱逐。
