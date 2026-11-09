# Agent 与 AgentState：能力复用，会话独立

`Agent` 持有**能力**，`AgentState` 持有**一段对话**。分离它们比为每种工作流再派生一种 Agent 类型更加重要。

## 复用一个 Agent

```rust
let agent = Agent::make(engine_config).await?
    .preamble("你是一个简洁的助手。")
    .max_iterations(8);

let runner = ChatRunner::default();
let alice = AgentState::new_shared("conversation-alice");
let bob = AgentState::new_shared("conversation-bob");

let a = runner.chat(&agent, &alice, "你好").await?;
let b = runner.chat(&agent, &bob, "你好").await?;
```

两份 State 的历史互不共享。`Agent` 主要依靠 `Arc` 共享资源，克隆成本较低。尽量使用不同的 `session_id`；本地 llama.cpp 会在推理状态缓存管理中利用它区分会话。

## Prompt 与模板

`Agent::preamble(&str)` 设置可复用的系统提示词；`.template(ChatTemplateType::Chatml)` 选择内置聊天模板。不同模型可能需要不同格式，也可以使用自定义 `ChatTemplate`。

```rust
use ambi::types::ChatTemplateType;
let agent = Agent::make(engine_config).await?
    .template(ChatTemplateType::Chatml)
    .preamble("给出依据，不确定时明确说明。");
```

## 动态上下文属于会话

```rust
{
    let mut session = alice.write().await;
    session.set_dynamic_context("参考资料：项目设计文档 v3");
    session.append_dynamic_context("语言：简体中文");
}

// 信息过期后，显式清理。
alice.write().await.clear_dynamic_context();
```

动态上下文被组合进模型的有效 System Context，不是普通聊天历史，**不会自动通过 FIFO 历史驱逐机制裁剪**。应用需要负责刷新与清理它。

## fork 与重新开始

`AgentState::fork()` 可以复制会话分支，底层历史消息内容通过 `Arc` 共享，适用于 ToT 等多路径探索。`AgentState::new_shared()` 则是全新的会话。

清理历史：

```rust
let mut session = alice.write().await;
ChatRunner::clear_history(&agent, &mut session);
```

**注意：** `clear_history` 还会调用共享 Engine 的 `reset_context()`。如果多个会话共用一个本地引擎，这个 Reset 并非只影响当前会话。

## 并发由应用管理

`Arc<RwLock<AgentState>>` 保证单次 Rust 内存读写的同步，却**不保证一个完整聊天请求的串行化**。需要严格会话顺序时，在使用同一 State 的调用外层建立 Session Mutex 或队列。不同 State 可以复用同一个 Agent。

## 错误语义

公共接口返回 `Result<_, AmbiError>`。需要时应单独处理 Engine、Tool、Pipeline 相关错误和 `MaxIterationsReached`。一次失败的 Run 可能已经发生不可逆的工具操作，不能把错误等同于「什么都没发生」。

继续：[架构](/zh/advanced/architecture) · [工具](/zh/guide/tools) · [Leaf](/zh/guide/leaf)。
