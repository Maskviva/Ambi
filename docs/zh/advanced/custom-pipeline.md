# 自定义 Pipeline：定义执行，而不是继承 Agent

`Pipeline` 定义**一次请求如何被执行**。内置 `ChatRunner` 实现 ReAct 风格的模型／工具循环，但第三方 Pipeline 不必采用同一策略。执行策略与 `Agent` 分离，是 Ambi 组合设计的核心边界。

建议先阅读 [设计理念](/zh/advanced/design-philosophy) 和 [架构](/zh/advanced/architecture)。

## 公共接口

原生平台上的 `Pipeline` Trait 需要实现两个方法：

```rust
use ambi::agent::pipeline::Pipeline;
// execute(&self, &Agent, &Arc<RwLock<AgentState>>, Vec<ContentPart>) -> Future<Result<String>>
// execute_stream(...) -> Future<Result<Pin<Box<ReceiverStream<Result<String>>>>>>
```

Trait 实际采用 `impl Future` 返回类型，原生平台需要 `Send + Sync`。两个方法都需要提供；如果不支持流式执行，可以像现有 Reflexion 一样返回 `PipelineError`，但应明确告知调用方。

## 先从组合已有 ChatRunner 开始

比复制内部实现更稳妥的方式，是包装公开的 `Pipeline` 接口：

```rust
use ambi::agent::pipeline::Pipeline;
use ambi::error::Result;
use ambi::{Agent, AgentState, ChatRunner, ContentPart};
use std::{pin::Pin, sync::Arc};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

struct MyPipeline {
    inner: ChatRunner,
}

impl Pipeline for MyPipeline {
    async fn execute(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<String> {
        // 在此增加前置/后置策略。
        self.inner.execute(agent, state, input).await
    }

    async fn execute_stream(
        &self,
        agent: &Agent,
        state: &Arc<RwLock<AgentState>>,
        input: Vec<ContentPart>,
    ) -> Result<Pin<Box<ReceiverStream<Result<String>>>>> {
        self.inner.execute_stream(agent, state, input).await
    }
}
```

此例需要额外依赖 `tokio-stream = "0.1"`。随后构造 `MyPipeline { inner: ChatRunner::default() }`，并调用 `pipeline.execute(...)`。如果无额外策略，直接用 ChatRunner 即可。

## 实现完全不同的执行循环

自定义 Pipeline 可以通过 `agent.llm_engine()` 取得共享的 `Arc<LLMEngine>`，自行构造 `LLMRequest` 和状态／工具执行逻辑。真正不同的执行方式意味着开发者需要负责历史写入、请求构造、流式交付和工具策略。`ChatRunner::run_loop` 目前**不是**对外公开的可复用原语。

**不要假定所有 Pipeline 已经可以任意互相嵌套。** `StandardRagPipeline`、`ReflexionPipeline` 当前都持有内部 `ChatRunner`，并非接受任意 `P: Pipeline` 的泛型包装器。Ambi 目前也不是 DAG 调度引擎。

## 为什么 Leaf 属于 ChatRunner

0.4.1 的 Leaf 使用 ChatRunner 的内部 ReAct 循环，仅在调用 `.with_leaf()` / `.with_leaf_config(...)` 时作为工具注入。开启 Cargo Feature 本身不改变默认 Runner，也不会让其他 Pipeline 悄悄获得 Leaf。

其他 Pipeline 完全可以定义自己的临时任务策略，而不需要遵守一个预先设计的公共 `Leaf` Trait。参见 [Leaf](/zh/guide/leaf)。

## 实现前的检查

- 明确共享配置和会话状态分别由谁拥有。
- 明确同步和流式两种行为，必要时显式报不支持。
- 同一 State 并发执行时，在应用层协调顺序。
- 明确工具权限、超时重试以及外部副作用的语义。
- 避免把不必要的内部过程全部塞进父级会话历史。

更多具体策略见 [`ambi-pipelines`](/zh/extensions/ambi-pipelines)。
