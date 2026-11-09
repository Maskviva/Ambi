# ambi-pipelines：可选执行策略

`ambi-pipelines` 将高阶认知策略放在 Ambi 小型核心之外。在 Ambi 0.4.1 仓库中，其源码版本为 **0.1.1**。这些 Pipeline 实现统一 `Pipeline` 接口，但**不是**任意可嵌套的图节点。

```toml
[dependencies]
ambi = "0.4.1"
ambi-pipelines = "0.1.1"
```

如果 Registry 未发布匹配版本，可以使用同一 Workspace 的 `path` 依赖。

## 内置策略

| Pipeline | 执行方式 | 需要注意 |
| --- | --- | --- |
| `StandardRagPipeline` | 检索、打包文档，写入动态上下文，再交给内部 ChatRunner | 会修改 Session 的动态上下文 |
| `SelfConsistencyPipeline` | 多分支执行并聚合答案 | 暂不支持流式接口 |
| `BfsBeamSearchPipeline` | 扩展、评估与筛选思考分支 | 需开发者提供 Expander / Evaluator |
| `ReflexionPipeline` | Actor/Evaluator 循环，利用 KV Provider 记录评语 | 暂不支持流式接口 |

## 示例：RAG Pipeline

```rust
use ambi::agent::pipeline::Pipeline;
use ambi_pipelines::rag::StandardRagPipeline;

// my_retriever 实现 ambi_pipelines::rag::Retriever。
let pipeline = StandardRagPipeline::create(my_retriever)
    .max_context_tokens(4096)
    .concurrency(3);

let answer = pipeline.execute(&agent, &state, input_parts).await?;
```

`Retriever`、`Document` 属于 `ambi-pipelines`，不是核心 `ambi`。RAG 会把检索片段注入 `AgentState.dynamic_context`，再交给内部 `ChatRunner`；应用应处理上下文新鲜度与共享 State 的并发问题。

## 为什么放在独立 Crate

Ambi 提供执行基础，应用选择认知策略。内置用例不应把完整思维框架强制加到只想使用小型 ReAct 循环或自定义 Pipeline 的开发者身上。

**Leaf 的范围：** 0.4.1 Leaf 只能显式挂载到 `ChatRunner`；这些扩展可能内部创建 `ChatRunner::default()`，即使编译了 Cargo Feature，默认仍无 Leaf。参见 [Leaf](/zh/guide/leaf) 与 [自定义 Pipeline](/zh/advanced/custom-pipeline)。
