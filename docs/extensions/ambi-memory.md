# ambi-memory: storage policy is an extension

`ambi-memory` is intentionally separate from Ambi's core. The runtime manages conversation state and FIFO eviction; **your application chooses whether and where to store lasting knowledge**.

The companion crate's source version is `0.1.1` in the Ambi 0.4.1 repository.

```toml
[dependencies]
ambi = "0.4.1"
ambi-memory = "0.1.1"
```

If registry versions are not available or not aligned, use matching workspace `path` dependencies.

## Three kinds of memory contracts

| Contract | Purpose | Built-in implementation |
| --- | --- | --- |
| `KvMemoryProvider` | Exact session-scoped key-value entries | `InMemoryKvProvider` |
| `SummaryMemoryProvider` | Store and retrieve summaries of prior context | `InMemorySummaryProvider` |
| `SemanticMemoryProvider` | Store/search semantic memories | **Trait only**; supply a provider |

The `InMemory*` implementations are process-local; they do not give database durability. The crate does **not** ship a ready-made vector database or embedding backend.

## Key-value example

```rust
use ambi::AgentState;
use ambi_memory::{AgentStateMemoryExt, InMemoryKvProvider};

let provider = InMemoryKvProvider::new();
let mut state = AgentState::new("session-1");
state.remember_kv(&provider, "project", "Ambi").await?;
state.recall_kv_into_context(&provider).await?;
```

Recalling into dynamic context is explicit. The caller is responsible for clearing or refreshing stale context; the memory extension does not silently own the `Agent` lifetime.

## Summaries and eviction

An application can use `on_evict` to observe removed messages and store summaries through its own provider and scheduling logic. This must be integrated deliberately: the core's eviction callback is not a durable background job system.

Leaf's temporary state does not use the parent's eviction persistence callback. A Leaf therefore does **not** automatically add its full history to long-term memory; only its returned result is visible to the parent.

See [Context eviction](/advanced/context-eviction) · [Leaf](/guide/leaf).
