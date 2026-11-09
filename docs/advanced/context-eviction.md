# Context eviction: a budget, not a guarantee

Ambi keeps session history distinct from shared `Agent` configuration. `EvictionStrategy` supplies a `max_safe_tokens` threshold (default **8,000**) to limit retained chat history.

## How the current algorithm works

`ChatHistory` stores `(Arc<Message>, token_count)` entries and maintains a cached `total_tokens`. When history plus prompt overhead exceeds the configured threshold, it removes messages **from the front (FIFO)** until the threshold is met or there are no more messages.

```rust
use ambi::config::EvictionStrategy;

let agent = Agent::make(engine_config).await?
    .with_eviction_strategy(EvictionStrategy { max_safe_tokens: 4096 });
```

System Prompt and `AgentState.dynamic_context` are not normal history messages. Their overhead is assessed separately. `on_evict` callbacks can be used to record removed messages, but storage is an application concern, not guaranteed by the core.

## Real limitations

- The default tokenizer is an **approximate accounting mechanism**, not authoritative exact tokenization for every backend. Local llama.cpp exposes a separate exact token-counting path.
- Eviction works at the **individual-message** level; it does not atomically preserve assistant/tool-call/result groups.
- Context budgets do not inherently cover all multimodal and provider-specific token costs.
- Large dynamic context or an oversize single input can still exceed a model's supported window.
- Eviction is not the same as long-term memory. For pluggable memory providers, see [`ambi-memory`](/extensions/ambi-memory).

Treat `max_safe_tokens` as a tuning parameter, not as a proof that requests can never fail with a context-length error.

## Leaf and eviction

Each Leaf invocation has a fresh history and inherits the parent's eviction strategy. Its state is discarded at completion, and its internal eviction does **not** call the parent's persistence callback. Only its final answer is returned to the parent.

This is a deliberate context-lifecycle choice, not a replacement for durable storage. More: [Leaf](/guide/leaf) · [Architecture](/advanced/architecture).
