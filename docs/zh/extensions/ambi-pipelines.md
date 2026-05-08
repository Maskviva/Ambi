# ambi-pipelines

Ambi AI 智能体框架的高级认知执行管道。

```toml
[dependencies]
ambi-pipelines = "0.1"
```

本 crate 中的所有管道均实现了 `Pipeline` trait，可以作为默认 `ChatRunner` 的即插即用替代：

```rust
use ambi_pipelines::rag::StandardRagPipeline;

let pipeline = StandardRagPipeline::create(my_retriever);

// 可在任何需要 Pipeline 的地方使用
let reply = pipeline.execute(&agent, &state, input).await?;
```

## RAG——检索增强生成

检索相关文档，按 token 预算打包进上下文，然后委托给内部的聊天循环。

```rust
use ambi_pipelines::rag::{StandardRagPipeline, Retriever};
use async_trait::async_trait;

struct MyRetriever;

#[async_trait]
impl Retriever for MyRetriever {
    async fn retrieve(&self, query: &str)
        -> Result<Vec<Document>, Box<dyn Error + Send + Sync>>
    {
        Ok(vec![Document::new("doc-1", "Ambi 是一个 AI 框架。", 0.95)])
    }
}

let pipeline = StandardRagPipeline::create(MyRetriever)
    .max_context_tokens(4096)
    .concurrency(5);
```

`ContextPacker` 按分数降序排列文档，在超出预算前停止，并为 LLM 附加引用 ID。

### SemanticMemoryRetriever

一个预构建的 `Retriever`，包装了 `ambi-memory` 的 `SemanticMemoryProvider`：

```rust
use ambi_pipelines::rag::SemanticMemoryRetriever;
use ambi_memory::SemanticMemoryProvider;

let retriever = SemanticMemoryRetriever::new(my_memory_provider, "session-1", 5);
let pipeline = StandardRagPipeline::create(retriever);
```

## 思维链自一致性（Self-Consistency）

并行运行多个独立的推理分支，从每个分支中提取规范答案，通过多数投票选出胜者。

```rust
use ambi_pipelines::cot::SelfConsistencyPipeline;
use ambi_pipelines::cot::PatternExtractor;

let pipeline = SelfConsistencyPipeline::create(
    PatternExtractor::new("答案：", "\n")
)
.branches(5)        // 5 条并行推理路径
.concurrency(5);
```

## 思维树 BFS 束搜索

通过广度优先束搜索探索多步推理——展开候选思维、评估、剪枝低分分支、保留 Top-B 节点。

```rust
use ambi_pipelines::tot::BfsBeamSearchPipeline;
use ambi_pipelines::tot::{TotExpander, TotEvaluator, Evaluation};

let pipeline = BfsBeamSearchPipeline::create(MyExpander, MyEvaluator)
    .branching_factor(3)   // 每节点 K 个候选思维
    .beam_width(2)         // 每层保留 B 个节点
    .max_steps(5);         // 最大探索深度
```

## Reflexion——执行者-评估者循环

生成响应、评估是否达标、将批评写入持久化 KV 记忆、然后重试，直到通过或耗尽重试次数。

```rust
use ambi_pipelines::reflexion::ReflexionPipeline;
use ambi_pipelines::reflexion::{Evaluator, EvaluationResult};
use ambi_memory::InMemoryKvProvider;

let pipeline = ReflexionPipeline::create(MyEvaluator, InMemoryKvProvider::new())
    .max_retries(3);
```
