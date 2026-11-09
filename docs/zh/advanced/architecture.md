# 架构：能力、状态与执行分离

Ambi 的核心刻意保持小型。它不是工作流图引擎，也不需要多智能体注册中心。核心只有三个需要分清的角色：可复用的 **Agent**、由调用方持有的 **AgentState**、可替换的 **Pipeline**。

建议先读 [设计理念](/zh/advanced/design-philosophy)，再阅读这个架构说明；[Leaf](/zh/guide/leaf) 则展示其中一种可选能力。

## 三个基本边界

```text
                         Agent
                  Engine / 配置 / 工具
                    解析器 / 格式化器
                           │
                ┌──────────┴──────────┐
                │                     │
         ChatRunner: Pipeline       自定义 Pipeline
          （ReAct 工具循环）        （自定义执行策略）
                │                     │
                └──────────┬──────────┘
                           │
                        AgentState
                session_id / chat_history
                dynamic_context / extensions
```

`Agent` 内部通过 `Arc` 共享主要资源，克隆成本较低。`AgentState` 支持 `serde` 序列化，不过类型擦除的扩展映射不会被序列化。`AgentState::new_shared()` 返回 `Arc<RwLock<AgentState>>`。通常应当**每个会话创建自己的 State**。

同一个 Agent 可以服务不同会话，但 **`RwLock` 不等于整个 Run 的事务锁**：执行过程会分多次获取和释放锁。如果把同一个 State 交给并发请求，应用层需要自行保证对话顺序。

## ChatRunner 的一次请求

1. 校验输入，例如多模态支持和并发参数。
2. 解析工具集合；启用 Leaf 时，为父级的本次执行增广工具快照。
3. 将用户消息写入 `AgentState.chat_history`。
4. 根据 System Prompt、动态上下文、工具定义、历史、模板与图片构建 `LLMRequest`（含 `session_id`）。
5. 调用 Engine，解析模型输出中的工具请求。
6. 按当前循环的并发上限执行工具，并应用每个工具的超时和重试策略。
7. 将工具结果反馈给模型，直到产生最终结果或触发迭代上限。
8. 统计历史 Token，并根据预算驱逐较早的消息。

`chat()` 返回 `Result<String>`，`chat_stream()` 返回 `Result<String>` 的流。二者复用核心 ReAct 循环，但输出传递方式不同。

## Engine 边界

`LLMEngineTrait` 提供 `chat(LLMRequest) -> Result<String>`、`chat_stream(...)`、`reset_context()`、`supports_multimodal()` 等接口。它目前以字符串为主要输出，**尚不是完整的结构化模型事件协议**。

- **OpenAI 兼容 API**：HTTP 请求并发信号量、请求超时；图片能力按已知模型名前缀推断或手动指定。
- **llama.cpp**：仅原生目标平台；使用有界命令队列、推理取消检查和共享 KV Cache。`session_id` 防止错误复用跨会话缓存；交替会话可能重新 Prefill。
- **自定义后端**：通过 `LLMEngineConfig::Custom(Box<dyn LLMEngineTrait>)` 接入。

自定义 Pipeline 可通过公开的 `Agent::llm_engine()` 取得 `Arc<LLMEngine>`，无需访问私有字段。

## Leaf 复用机制，而不是身份

可选 Leaf 基于 `ChatRunner` 与 `Tool` 机制实现。父级看到原始工具加 `spawn_leaf`；Leaf 看到的原始工具快照中**没有** `spawn_leaf`。Leaf 以新 `AgentState` 运行 `run_loop`，采用 `final_answer_only = true`。

它不是另一个长期存在的 Agent；不会继承父级对话历史，但**共享 Engine 和父级工具**。参见 [Leaf 任务委派](/zh/guide/leaf)。

## Runtime 做到了什么、没有承诺什么

| 主题 | 当前机制 | 边界 |
| --- | --- | --- |
| 会话状态 | Agent 与 Session State 分离 | 不自动保证同一 State 的 Run 串行化 |
| 历史驱逐 | 基于近似 Token 统计的 FIFO 消息裁剪 | 不保证工具调用组原子裁剪或绝对不超窗 |
| 工具重试 | 仅标记幂等的工具允许超时重试 | 取消 Future 不等于撤销外部副作用 |
| Leaf | 临时状态，只返回最终答案 | 共享引擎与原始工具权限 |
| Pipeline | 开发者可实现 Trait | 未内置任意 DAG 组合 |
| 流式执行 | 通道驱动，支持取消敏感处理 | 外部请求仍有独立的取消语义 |

如果应用需要审计级工具事件、持久任务队列、严格权限隔离，应在这些基本机制之上显式构建。

## 源码索引

| 主题 | 源码路径 |
| --- | --- |
| Agent / State | `src/agent/core.rs`、`core/builder.rs` |
| Pipeline / ChatRunner | `src/agent/pipeline.rs`、`pipeline/chat_runner.rs` |
| ReAct 循环 | `src/agent/pipeline/chat_runner/multimodal_handler.rs` |
| Leaf | `src/agent/pipeline/chat_runner/leaf.rs` |
| 工具 | `src/types/tool_def.rs`、`src/agent/tool/manager.rs` |
| Engine | `src/llm/engine.rs`、`src/llm/providers/` |
| 历史 | `src/agent/core/history.rs` |

继续阅读：[自定义 Pipeline](/zh/advanced/custom-pipeline) · [自定义 Engine](/zh/advanced/custom-engine) · [上下文驱逐](/zh/advanced/context-eviction)。
