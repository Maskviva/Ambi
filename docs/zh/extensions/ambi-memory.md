# ambi-memory：持久化策略属于扩展

`ambi-memory` 与核心 Ambi 保持分离。Runtime 负责会话历史和 FIFO 驱逐；**要不要长期记忆、将知识保存在哪里，由应用决定**。

在 0.4.1 仓库中，伴随的 `ambi-memory` 源码版本为 `0.1.1`。

```toml
[dependencies]
ambi = "0.4.1"
ambi-memory = "0.1.1"
```

如 Registry 尚未发布对应版本或依赖不匹配，可使用同一仓库的 `path` 依赖。

## 三种记忆契约

| Trait | 职责 | 内置实现 |
| --- | --- | --- |
| `KvMemoryProvider` | 会话范围的精确 KV | `InMemoryKvProvider` |
| `SummaryMemoryProvider` | 保存与读取历史摘要 | `InMemorySummaryProvider` |
| `SemanticMemoryProvider` | 语义记忆存储与检索 | **仅 Trait**，需自建 Provider |

`InMemory*` 仅在进程内保存数据，不具备数据库级持久性。Crate **没有**开箱即用的向量数据库或 Embedding 后端。

## KV 示例

```rust
use ambi::AgentState;
use ambi_memory::{AgentStateMemoryExt, InMemoryKvProvider};

let provider = InMemoryKvProvider::new();
let mut state = AgentState::new("session-1");
state.remember_kv(&provider, "project", "Ambi").await?;
state.recall_kv_into_context(&provider).await?;
```

记忆注入动态上下文是显式操作。应用需要自行清理过期上下文；Memory 扩展不会隐式接管 `Agent` 的生命周期。

## 摘要与历史驱逐

可以通过 `on_evict` 观察被删除消息，并使用自己的 Provider 和任务调度逻辑生成摘要。但该回调本身不是持久后台任务系统。

Leaf 临时状态不使用父级的驱逐持久化回调，因此不会自动把完整内部历史写入长期记忆；父级仅接收返回结果。

参见 [上下文驱逐](/zh/advanced/context-eviction) · [Leaf](/zh/guide/leaf)。
