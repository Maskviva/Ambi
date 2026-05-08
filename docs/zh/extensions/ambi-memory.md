# ambi-memory

Ambi AI 智能体框架的可插拔三维记忆扩展。

```toml
[dependencies]
ambi-memory = "0.1"
```

## 三个记忆维度

### KV 记忆——精确状态存储

按会话存储和检索任意键值对。适用于反思日记、用户设置、状态机标记等需要精确查找的结构化数据。

```rust
use ambi_memory::{AgentStateMemoryExt, InMemoryKvProvider};

let kv = InMemoryKvProvider::new();
let mut state = AgentState::new("session-1");

state.remember_kv(&kv, "user_name", "Alice").await?;
state.recall_kv_into_context(&kv).await?;
// dynamic_context 现在包含："[STATE MEMORY]:\n- user_name: Alice\n"
```

### 语义记忆——向量化召回

按语义相似度存储和检索历史对话片段。适用于跨会话的长期记忆。通过实现 trait 对接你的向量数据库（Milvus, Qdrant, Pinecone 等）。

```rust
use ambi_memory::SemanticMemoryProvider;

async fn remember(provider: &dyn SemanticMemoryProvider, state: &mut AgentState) {
    state.archive_semantic(provider, "用户提到他们更喜欢 Python。").await?;
    state.recall_semantic_into_context(provider, "用户喜欢什么语言？", 3).await?;
}
```

### 摘要记忆——上下文驱逐的防遗忘机制

当 FIFO 上下文驱逐算法丢弃旧消息时，该提供者利用 LLM 自身将这些消息自动压缩为滚动摘要。

```rust
use ambi_memory::{AgentStateMemoryExt, InMemorySummaryProvider};

let summary_provider = InMemorySummaryProvider::new();

// 每次对话前，注入累积的摘要：
state.inject_summary_context(&summary_provider).await?;

// 在 on_evict 回调中，压缩被丢弃的消息：
state.summarize_evicted_messages(&agent, &summary_provider, &evicted).await?;
```

## 扩展 Trait

三个记忆维度通过统一的 `AgentStateMemoryExt` trait 暴露，该 trait 在 `AgentState` 上实现：

| 方法 | 维度 | 作用 |
|------|------|------|
| `remember_kv(provider, key, value)` | KV | 存储一个值 |
| `recall_kv_into_context(provider)` | KV | 将所有 KV 对注入 `dynamic_context` |
| `archive_semantic(provider, text)` | 语义 | 嵌入并持久化文本 |
| `recall_semantic_into_context(provider, query, limit)` | 语义 | 将 Top-N 相似记忆注入 `dynamic_context` |
| `inject_summary_context(provider)` | 摘要 | 将滚动摘要前置到 `dynamic_context` |
| `summarize_evicted_messages(agent, provider, evicted)` | 摘要 | 通过 LLM 压缩被驱逐的消息 |

## 后端实现

该库内置了**内存级**实现（`InMemoryKvProvider`, `InMemorySummaryProvider`），用于测试和单机部署。生产环境中，请针对你的数据库实现对应 trait。
