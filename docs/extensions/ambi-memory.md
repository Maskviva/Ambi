# ambi-memory

Pluggable three-dimensional memory extension for the Ambi AI agent framework.

```toml
[dependencies]
ambi-memory = "0.1"
```

## Three memory dimensions

### KV Memory — exact state storage

Store and retrieve arbitrary key-value pairs per session. Ideal for reflexion diaries, user settings, state machine flags, or any structured data that needs exact lookup.

```rust
use ambi_memory::{AgentStateMemoryExt, InMemoryKvProvider};

let kv = InMemoryKvProvider::new();
let mut state = AgentState::new("session-1");

state.remember_kv(&kv, "user_name", "Alice").await?;
state.recall_kv_into_context(&kv).await?;
// dynamic_context now contains: "[STATE MEMORY]:\n- user_name: Alice\n"
```

### Semantic Memory — vector-based recall

Store and search past conversation chunks by semantic similarity. Useful for long-term memory across multi-session interactions. Implement the trait against your preferred vector database (Milvus, Qdrant, Pinecone, etc.).

```rust
use ambi_memory::SemanticMemoryProvider;

async fn remember(provider: &dyn SemanticMemoryProvider, state: &mut AgentState) {
    state.archive_semantic(provider, "User mentioned they prefer Python.").await?;
    state.recall_semantic_into_context(provider, "What language does the user like?", 3).await?;
}
```

### Summary Memory — anti-amnesia on eviction

When the FIFO context eviction algorithm drops old messages, this provider automatically compresses them into a rolling summary using the LLM itself.

```rust
use ambi_memory::{AgentStateMemoryExt, InMemorySummaryProvider};

let summary_provider = InMemorySummaryProvider::new();

// Before each turn, inject the accumulated summary:
state.inject_summary_context(&summary_provider).await?;

// Inside an on_evict callback, compress what was dropped:
state.summarize_evicted_messages(&agent, &summary_provider, &evicted).await?;
```

## Extension trait

All three memory dimensions are exposed through a single `AgentStateMemoryExt` trait, which is implemented on `AgentState`:

| Method | Dimension | Effect |
|--------|-----------|--------|
| `remember_kv(provider, key, value)` | KV | Stores a value |
| `recall_kv_into_context(provider)` | KV | Injects all KV pairs into `dynamic_context` |
| `archive_semantic(provider, text)` | Semantic | Embeds and persists text |
| `recall_semantic_into_context(provider, query, limit)` | Semantic | Injects top-N similar memories into `dynamic_context` |
| `inject_summary_context(provider)` | Summary | Prepends the rolling summary to `dynamic_context` |
| `summarize_evicted_messages(agent, provider, evicted)` | Summary | Compresses dropped messages via the LLM |

## Backend implementations

The crate ships **in-memory** implementations (`InMemoryKvProvider`, `InMemorySummaryProvider`) for testing and single-node deployments. For production, implement the corresponding trait against your database:

```rust
struct MyKvProvider { /* connection pool */ }

#[async_trait]
impl KvMemoryProvider for MyKvProvider {
    async fn store(&self, session_id: &str, key: &str, value: &str) -> Result<()> { /* ... */ }
    async fn retrieve(&self, session_id: &str, key: &str) -> Result<Option<String>> { /* ... */ }
    async fn retrieve_all(&self, session_id: &str) -> Result<HashMap<String, String>> { /* ... */ }
    async fn clear_session(&self, session_id: &str) -> Result<()> { /* ... */ }
}
```
