# 自定义管道

管道是驱动 Agent 的执行循环。`ChatRunner` 是默认实现，包含完整的 ReAct 循环和工具处理。

## 什么时候需要写自定义管道

- 想要一个更简单的循环，不需要工具支持（纯对话）
- 需要不同的错误处理或重试逻辑
- 在做专门的 Agent（比如纯思维链，没有工具）

## 实现 Pipeline

```rust
use ambi::agent::pipeline::Pipeline;
use ambi::agent::core::{Agent, AgentState};
use ambi::error::Result;
use ambi::ContentPart;

struct MySimplePipeline;

impl Pipeline for MySimplePipeline {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        // 从当前状态构建请求
        let req = /* ... */;

        // 直接调引擎
        let response = agent.llm_engine.chat(req).await?;

        // 不处理工具——直接返回原始文本
        Ok(response)
    }

    async fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        // ... 流式实现
        unimplemented!()
    }
}
```

注意 `llm_engine` 是 `pub(crate)` 的，所以你只能在 crate 内部实现自定义管道，或者在外部组合 `ChatRunner` 的公开 API。

## 组合比替换更常见

大多数情况下你不需要替换整个管道——定制各个部件就够了：

- **格式化** → 自定义 `StreamFormatter`（见[流式格式化器](/zh/advanced/stream-formatter)）
- **工具解析** → 自定义 `ToolCallParser`（见[工具解析器](/zh/advanced/tool-parser)）
- **引擎行为** → 自定义 `LLMEngineTrait`（见[自定义引擎](/zh/advanced/custom-engine)）

如果真的需要完全自定义循环，复制 `ChatRunner` 的结构然后改你需要的部分。
