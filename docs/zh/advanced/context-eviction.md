# 上下文驱逐

长对话会消耗大量 token。Ambi 用确定性的 FIFO 驱逐算法来控制上下文预算。

## 工作原理

每条消息在 `ChatHistory` 里都带着精确的 token 数：

```rust
struct ChatHistory {
    messages: Vec<(Arc<Message>, usize)>,  // (消息, token 数)
    total_tokens: usize,
}
```

推送新助手消息时，驱逐检查会跑：

```
total_tokens + prompt_overhead > max_safe_tokens ?
    → 是：从最早的消息开始弹出，直到预算够用
    → 否：不动
```

驱逐是 FIFO 的：最早的消息先被移除。最近的对话不受影响。

```rust
// 核心算法，来自 history.rs：
pub fn evict_old_messages(&mut self, max_safe_tokens: usize, prompt_overhead: usize) -> Vec<Arc<Message>> {
    let mut target = self.total_tokens + prompt_overhead;
    let mut to_remove = 0;

    for (_, tokens) in &self.messages {
        target -= tokens;
        to_remove += 1;
        if target <= max_safe_tokens { break; }
    }

    self.messages.drain(0..to_remove)
}
```

## "prompt 开销"包括什么

- 系统提示词 token（来自 `AgentConfig`）
- 动态上下文 token（来自 `AgentState::dynamic_context`）
- 工具指令提示词 token（缓存在 `Agent::cached_tool_prompt`）

注意：`Message::System` 不再被推入 `ChatHistory`。历史记录是 `User`、`Assistant`、`Tool` 事件的纯 FIFO 队列，
确保 O(1) 截断和最大化 KV Cache 前缀匹配。

每轮迭代动态计算：

```rust
let prompt_overhead = engine.count_tokens(system_prompt)?
    + engine.count_tokens(&state.dynamic_context)?
    + engine.count_tokens(&agent.cached_tool_prompt)?;
```

## 配置阈值

```rust
use ambi::config::EvictionStrategy;

let agent = Agent::make(config).await?
    .with_eviction_strategy(EvictionStrategy { max_safe_tokens: 4096 });
```

### 怎么选值

默认是 8000。对 8K 上下文的模型，这留给输出大约 4K 的空间。128K 模型可以设到 64000 或更高。观察你的平均输出长度来调整。

## 驱逐回调

当消息被驱逐时可以注册一个钩子。回调现在接收 `&AgentState` 作为第一个参数，可以安全地从 state extensions
中提取会话标识符和连接池，用于异步数据库归档：

```rust
use ambi::{Agent, AgentState};
use std::sync::Arc;

let agent = Agent::make(config).await?
    .on_evict(|state: &AgentState, evicted: Vec<Arc<Message>>| {
        let session_id = &state.session_id;
        // 注意：在持有 AgentState 写锁时执行。
        // 对于 I/O 密集型操作，应使用 tokio::spawn：
        tokio::spawn(async move {
            // 将被驱逐的消息持久化到数据库
        });
    });
```

使用场景：
- **持久化** —— 把旧消息存到数据库里，后续可以检索
- **汇总** —— 把被驱逐的消息浓缩成摘要
- **审计日志** —— 追踪丢了什么

## 驱逐时机

驱逐在每轮 ReAct 迭代结束时运行，紧跟在助手消息被追加到历史之后。如果这轮产生了工具调用，工具消息会接着被推进历史，下一次 LLM 调用会再次触发驱逐检查。

## 安全限制

- `max_iterations`（默认 10）防止无限循环
- 达到最大迭代次数时，历史会回滚到请求开始前的快照
- 非幂等工具不会重试，防止驱逐导致的重复副作用
