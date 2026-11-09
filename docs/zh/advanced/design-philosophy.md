# 设计理念：小型 Runtime，开放的上限

Ambi 是一个**小而可组合的 Rust AI Agent Runtime**。它提供模型推理、工具执行、上下文管理和智能体执行循环等基础机制，但不规定所有智能体必须如何思考、规划或协作。

这是一条架构边界，而不是宣称“有了 Trait 就能自动组合任何工作流”。公共接口有明确的职责，也存在现实限制。目标是让核心始终可理解，并尽量让新行为通过组合扩展，而不是持续膨胀核心。

## 组合优于继承

Ambi 不追求 `BaseAgent`、`WorkerAgent`、`SupervisorAgent` 等类层级。开发者根据需求组合能力：

| 概念 | 职责 | 扩展方式 |
| --- | --- | --- |
| `Agent` | 可共享的 Engine、配置、工具、解析器与格式化器 | Builder 与组件组合 |
| `AgentState` | 会话历史与动态上下文 | 独立会话，或显式 `fork` |
| `Pipeline` | 决定请求如何执行 | 实现自己的执行策略 |
| `LLMEngineTrait` | 定义模型生成完整结果与流式结果的方式 | 内置或自定义后端 |
| `Tool` | 提供类型化的外部操作 | 应用开发者实现 |
| `ToolCallParser`、`StreamFormatter`、`TokenizerTrait` | 协议处理与展示边界 | 替换具体实现 |

**Ambi 的内置用例决定下限，而不应该规定上限。** 第三方可以实现 Ambi 作者没有预想过的 `Pipeline`，而无需将那套策略纳入核心 Crate。

但应区分：**可替换并不等于任意 Pipeline 都能直接嵌套组合**。例如目前 `StandardRagPipeline` 内部持有 `ChatRunner`，并不是泛型的 `Rag<P>` 包装器。Ambi 目前也不是通用图工作流引擎。

## 树、枝、叶：关于执行关系的比喻

```text
                                Agent
                        （共享模型、工具、配置）
                                  │
                     ┌────────────┴────────────┐
                     │                         │
                 ChatRunner                自定义 Pipeline
                 （ReAct 枝）                （另一条枝）
                     │
               可选 spawn_leaf
                     │
               ┌─────┴─────┐
               │           │
             Leaf A       Leaf B
          （临时状态）   （临时状态）
               │           │
             最终结果     最终结果

           AgentState 独立于 Agent，交给 Pipeline 使用。
```

**Agent 是树，Pipeline 是枝，Leaf 是叶。** 这是一种**设计比喻**，不是 Rust 继承体系，也不要求所有 Pipeline 都必须生成 Leaf。不同的枝可以产生不同的能力；开发者也可以通过 Trait 嫁接自己的实现。

为什么不叫 Sub-agent？仅仅因为某个任务由父级调用，并不代表它必须拥有一个长期存在的完整 `Agent` 身份。在 Ambi 0.4.1 中，Leaf 是挂载在 `ChatRunner` 工具集合上的**一次性执行**：独立创建临时会话，完成任务后只返回收敛的结果，随后丢弃。这样，任务过程中的大段上下文不会直接填满父级对话历史。

**「用完即丢」说的是执行身份与会话状态，不是外部世界无副作用。** 当前 Leaf 共享父级引擎，继承其已注册工具，因此仍可能执行有副作用的操作。详见 [Leaf 任务委派](/zh/guide/leaf)。

## Runtime 管机制，应用决定策略

Runtime 必须认真承担模型调用、工具超时和重试、流式处理、上下文历史统计、错误传递等机制责任。应用负责选择 System Prompt、执行策略、工具权限、持久化方式和业务语义。

这也是 `ambi-pipelines`、`ambi-memory` 和 `ambi-macros` 保持为独立 Crate 的原因：高级能力不应变成所有使用者必须接受的认知架构。

保持简洁不意味着模糊边界：

- 历史驱逐使用的 Token 统计是**近似值**，不能保证所有模型都永不超过上下文窗口。
- `AgentState` 与 `Agent` 分离，但**同一个可变 State 的并发 Run** 并不会天然形成完整会话级串行化。
- 工具 Future 超时或取消，**不代表外部操作已经撤销**。
- 本地 `llama.cpp` 引擎由共享 Worker 与 KV Cache 服务，Leaf 之间可能产生资源竞争。

代码层面的结构参见 [架构](/zh/advanced/architecture)，边界参见 [上下文驱逐](/zh/advanced/context-eviction)。

## 增加抽象前的判断原则

1. 优先尝试组合已有的 Engine、Pipeline、State 与 Tool。
2. 只有真实需求无法由现有契约表达时，再考虑增加核心抽象。
3. 可选能力不得在不知情的情况下改变现有 Pipeline 的默认行为。
4. 对取消、生命周期与外部副作用应说明真实保证，不靠名称制造安全感。

0.4.1 的 Leaf 正遵循这样的思路：Cargo Feature 和运行时配置**双重显式开启**；复用已有工具机制和 ReAct 循环；不额外引入 Agent Registry。其他 Pipeline 是否需要另一种 Leaf，由其作者决定。

## Ambi 不打算成为什么

Ambi 不以收集尽可能多的规划算法、角色体系、图工作流、持久调度器、权限系统或数据库为目标。小型 Runtime 可以让开发者实现这些东西，但**允许扩展**不等于**必须内置**。

Ambi 想做的，是一个**稳定、清晰、可被组合的执行基础**，而不是最长的功能清单。
