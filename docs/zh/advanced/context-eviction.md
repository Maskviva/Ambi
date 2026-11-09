# 上下文驱逐：预算，而不是绝对保证

Ambi 将会话历史与共享 `Agent` 配置分离。`EvictionStrategy` 的 `max_safe_tokens` 阈值（默认 **8,000**）用来限制保留的聊天历史规模。

## 当前算法

`ChatHistory` 使用 `(Arc<Message>, token_count)` 保存消息，并缓存 `total_tokens`。当历史估算 Token 加上 Prompt 开销超过阈值时，算法按**消息级 FIFO** 从最老消息开始删除，直到满足预算或无消息可删。

```rust
use ambi::config::EvictionStrategy;

let agent = Agent::make(engine_config).await?
    .with_eviction_strategy(EvictionStrategy { max_safe_tokens: 4096 });
```

System Prompt 和 `AgentState.dynamic_context` 不属于普通历史消息，其开销在其他位置计算。`on_evict` 回调可接收被删除的消息，但持久化存储是应用层的责任，并不是核心自动提供的保证。

## 实际边界

- 默认 Tokenizer 主要用于**近似预算统计**，不等于所有模型的精确 Token 数。本地 llama.cpp 另外提供精确计算路径。
- 当前驱逐以**单条消息**为单位，并不保证 assistant tool call 与 tool result 被当作原子组一起保留。
- 多模态和各 Provider 特有的 Token 成本可能无法被预算完全覆盖。
- 过长的动态上下文或单条输入仍可能超过模型窗口。
- 驱逐不等于长期记忆。可插拔 Provider 参见 [`ambi-memory`](/zh/extensions/ambi-memory)。

因此，`max_safe_tokens` 是可调优的安全阈值，**不是**“永远不会出现上下文长度错误”的证明。

## Leaf 与上下文驱逐

每个 Leaf 都有独立的新历史并继承父级驱逐策略；完成后丢弃 State，且内部驱逐不会触发父级持久化回调。返回父级的只有最终结果。

这是一种有意设计的上下文生命周期策略，不是持久化存储的替代品。参见 [Leaf](/zh/guide/leaf) · [架构](/zh/advanced/architecture)。
