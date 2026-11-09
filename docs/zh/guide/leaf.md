# Leaf：用完即丢的任务委派

Leaf 在 **Ambi 0.4.1** 中提供，需要 `sub-agent` Cargo Feature。它属于 **`ChatRunner` 的显式可选能力**，不是一个独立的 `Agent` 类型，也不会自动影响其他 Pipeline。

## 为什么叫 Leaf？

父级智能体可能需要阅读大段项目代码、生成长文档摘要，但不应该把每次分析的中间过程全部放进自己的对话历史。传统做法可能是创建另一个完整 Agent；Ambi 则把这种需求理解为：**Pipeline 为一次任务生成的临时执行**。

在树的比喻中，Agent 是树，Pipeline 是枝，Leaf 是叶：因任务而生，完成后即丢弃。不同的枝可以有不同的叶。详细理念参见 [设计理念](/zh/advanced/design-philosophy)。

## 两步启用

**第一步：编译 Cargo Feature。**

```toml
[dependencies]
ambi = { version = "0.4.1", features = ["sub-agent"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

**第二步：在需要的 ChatRunner 上显式开启。**

```rust
use ambi::ChatRunner;

let runner = ChatRunner::default().with_leaf();
// Agent、AgentState 的创建方式与普通聊天保持一致。
```

仅开启 Cargo Feature 不会改变 `ChatRunner::default()`。默认向父级模型暴露的工具名为 `spawn_leaf`。

## 定制委派策略

```rust
use ambi::{ChatRunner, LeafConfig};

let cfg = LeafConfig::new()
    .with_tool_name("delegate_summary")
    .with_description("独立阅读指定文档或模块，只返回总结结论。")
    .with_system_prompt("你是代码研究助手。只报告可验证的发现与不确定性。")
    .with_max_iterations(6)
    .with_maximum_concurrency(2)
    .with_timeout_secs(120);

let runner = ChatRunner::default().with_leaf_config(cfg);
```

`LeafConfig` 可以覆盖系统提示词、迭代次数和 Leaf 内部工具并发数；未设置时继承父级值。默认超时是 300 秒。Leaf 继承父级模板、上下文驱逐策略与原始工具。若自定义工具名与现有工具冲突，框架会报错。

## 一次委派到底发生了什么？

```text
父级 LLM
   -> spawn_leaf({ "task": "总结模块 X", "context": "可选背景" })
       -> 创建全新 AgentState，不带父级对话历史
       -> 共享 Engine，获取原始工具快照
       -> 运行同一个内部 ReAct 循环
       -> 只返回最终答案
       -> 丢弃临时状态
   -> 父级把答案作为工具结果接收
```

模型可填写必需的 `task` 和可选的 `context`。`context` 会写入 Leaf 的动态上下文。由于不继承父级对话历史，父级需要明确传递任务真正需要的背景。

工具快照在注入 `spawn_leaf` **之前**取得：父级可以调用 Leaf，Leaf 自己看不到 `spawn_leaf`，因此无需深度计数器就可以结构性阻止递归。

Leaf 采用 `final_answer_only` 模式，避免把内部每一轮的文本、工具展示块全部返回给父级。它节约的是**父级上下文**，并不意味着 Leaf 内部推理不消耗 Token。

## 必须理解的边界

| 维度 | 0.4.1 的实际语义 |
| --- | --- |
| 会话历史 | 每次独立创建 `AgentState`，任务完成后丢弃 |
| 模型资源 | **共享**父级 Engine，并非独立模型实例 |
| 工具权限 | 继承父级**全部原始工具**，目前没有内置 Allow-list |
| 递归 | Leaf 的工具快照中不含委派工具，无法再次生成 Leaf |
| 外部副作用 | **不会自动撤销**；写文件、发邮件等仍可能生效 |
| 并发限制 | 每个 Leaf 循环独立限制，**不是**全局资源预算 |
| 本地 llama.cpp | 共享 Worker/KV Cache，可能排队并重新 Prefill |
| 超时 | 默认 300 秒；超时不等于外部调用已经撤销 |
| 持久化 | 临时历史不进入父级驱逐回调 |

如需隔离凭据、限制危险操作，应另外设计 Agent/工具权限边界。**Leaf 不是沙箱。** 流式父级也可以调用 Leaf，但 Leaf 内部采用同步执行循环，最后返回一个结果。

## 不适合 Leaf 的场景

需要独立持久身份、断点恢复、严格权限沙箱、由模型选择独立 Engine，或者任意递归多 Agent 编排时，当前 Leaf 并不提供这些承诺。

更多细节见 [工具](/zh/guide/tools) 与 [架构](/zh/advanced/architecture)。
