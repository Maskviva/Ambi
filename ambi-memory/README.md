# ambi-memory

Pluggable three-dimensional memory extension for the [Ambi](https://github.com/Maskviva/Ambi) AI agent framework.

## Dimensions

| Provider | Description |
|---|---|
| **KV (Key-Value)** | Exact state storage for reflexion diaries, user settings, and state machine flags. |
| **Semantic (Vector)** | Long-term conversational recall via embedding + vector search (e.g. OpenAI + Milvus/Qdrant). |
| **Summary (Rolling)** | Anti-amnesia on context eviction — automatically compresses dropped messages into a persistent summary. |

## Usage

```toml
[dependencies]
ambi-memory = "0.1"
```

```rust
use ambi_memory::{
    AgentStateMemoryExt,
    InMemoryKvProvider, KvMemoryProvider,
    InMemorySummaryProvider, SummaryMemoryProvider,
    SemanticMemoryProvider,
};
```

### KV Memory

```rust
let kv = InMemoryKvProvider::new();
let mut state = AgentState::new("session-1");

// Store
state.remember_kv(&kv, "user_name", "Alice").await?;

// Recall into dynamic context
state.recall_kv_into_context(&kv).await?;
```

### Summary Memory (auto-compression on eviction)

```rust
let summary_provider = InMemorySummaryProvider::new();
let mut state = AgentState::new("session-1");

// Inject past summary into context
state.inject_summary_context(&summary_provider).await?;

// Compress evicted messages (call inside on_evict handler)
state.summarize_evicted_messages(&agent, &summary_provider, &evicted).await?;
```

### Semantic Memory

```rust
// Use with any backend implementing SemanticMemoryProvider
async fn recall(provider: &dyn SemanticMemoryProvider, state: &mut AgentState, query: &str) {
    state.recall_semantic_into_context(provider, query, 5).await.unwrap();
}
```

## Feature Flags

All providers are **trait-only** — the crate ships in-memory implementations for testing and single-node deployments. Swap the backend by implementing the corresponding trait against your database (Milvus, Redis, PostgreSQL, etc.).

## License

Apache-2.0
