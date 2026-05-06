# Context Eviction

Long conversations eat up tokens. Ambi uses a deterministic FIFO eviction algorithm to keep the context within budget.

## How it works

Each message in `ChatHistory` is stored alongside its exact token count:

```rust
struct ChatHistory {
    messages: Vec<(Arc<Message>, usize)>,  // (message, token_count)
    total_tokens: usize,
}
```

When a new assistant message is pushed, the eviction check runs:

```
total_tokens + prompt_overhead > max_safe_tokens ?
    → YES: pop oldest messages until under budget
    → NO:  do nothing
```

The eviction is FIFO: oldest messages are removed first. This keeps recent conversation intact.

```rust
// Core algorithm from history.rs:
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

## What counts as "prompt overhead"

The overhead includes:
- System prompt tokens (from `AgentConfig`)
- Dynamic context tokens (from `AgentState::dynamic_context`)
- Tool instruction prompt tokens (cached in `Agent::cached_tool_prompt`)

Note: `Message::System` is no longer pushed into `ChatHistory`. The history is a pure FIFO queue
of `User`, `Assistant`, and `Tool` events, ensuring O(1) truncation and maximum KV Cache prefix matching.

This is computed dynamically per iteration:

```rust
let prompt_overhead = engine.count_tokens(system_prompt)?
    + engine.count_tokens(&state.dynamic_context)?
    + engine.count_tokens(&agent.cached_tool_prompt)?;
```

## Configuring the threshold

```rust
use ambi::config::EvictionStrategy;

let agent = Agent::make(config).await?
    .with_eviction_strategy(EvictionStrategy { max_safe_tokens: 4096 });
```

### Choosing a value

The default is 8000. For a model with 8K context, this leaves room for a ~4K output without hitting the limit. For a 128K model, 64000 might be reasonable. Monitor your average output length and adjust.

## Eviction callback

You can register a hook that fires whenever messages are evicted. The callback now receives
`&AgentState` as its first argument, allowing safe access to session identifiers and connection
pools from state extensions for async database archiving:

```rust
use ambi::{Agent, AgentState};
use std::sync::Arc;

let agent = Agent::make(config).await?
    .on_evict(|state: &AgentState, evicted: Vec<Arc<Message>>| {
        let session_id = &state.session_id;
        // NOTE: Runs while holding the AgentState write lock.
        // Spawn an async task for I/O-heavy operations:
        tokio::spawn(async move {
            // persist evicted messages to DB
        });
    });
```

Use cases:
- **Persistence** – save old messages to a database for retrieval later
- **Summarization** – condense evicted messages into summaries
- **Logging/audit** – track what was dropped

## When eviction happens

Eviction runs at the end of each ReAct iteration, just after the assistant message is appended to history. If the iteration produces tool calls, those tool messages go into history next, and the next LLM call will trigger another eviction check if needed.

## Safety limits

- `max_iterations` (default 10) prevents infinite loops
- If max iterations is reached, the history is rolled back to the snapshot taken before the request started
- Non-idempotent tools are not retried, preventing duplicate side effects from eviction-related re-runs
