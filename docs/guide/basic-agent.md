# Basic Agent

Once you have a running agent (see [Getting Started](/guide/getting-started)), this page covers the common patterns you'll use every day.

## System prompt

The system prompt sets the agent's behavior. Every message to the LLM includes it as a preamble.

```rust
let agent = Agent::make(config).await?
    .preamble("You are a concise assistant that answers in haiku.");
```

There is no limit on how long the preamble can be, but remember it counts toward your token budget.

## Chat templates

Different LLMs expect different prompt formats. Ambi supports 9 templates out of the box:

| Template    | Typical models                       |
|-------------|--------------------------------------|
| `Chatml`    | OpenAI, Qwen, many fine-tunes        |
| `Llama3`    | Llama 3 / 3.1 family                 |
| `Deepseek`  | DeepSeek V2/V3, R1                   |
| `Qwen`      | Qwen 2 / 2.5                         |
| `Gemma`     | Google Gemma                         |
| `Phi3`      | Microsoft Phi-3 / Phi-4              |
| `Mistral`   | Mistral / Mixtral                    |
| `Zephyr`    | HuggingFace Zephyr                   |
| `Llama2`    | Legacy Llama 2                       |

```rust
use ambi::ChatTemplateType;

let agent = Agent::make(config).await?
    .template(ChatTemplateType::Deepseek);
```

Default is `Chatml`. If your model requires a custom format, you can build a `ChatTemplate` struct manually.

## Multi-turn conversation

The agent keeps history in `AgentState`. Each call to `chat()` appends the user message and the assistant response to the state. The next call sees the full context.

```rust
let state = AgentState::new_shared("session-001");

runner.chat(&agent, &state, "My name is Alice.").await?;
// The assistant remembers "Alice" in the next turn:
runner.chat(&agent, &state, "What's my name?").await?;
// -> "Alice"
```

### session_id

Every `AgentState` carries a unique `session_id` that enables distributed tracing and KV cache slotting
for high-concurrency environments. Choose meaningful identifiers for your use case:

```rust
let state = AgentState::new_shared("user-42-conversation-3");
```

### Dynamic context

Injection of volatile data (RAG results, timestamps, environment variables) is done through `AgentState`
without polluting the static `system_prompt`:

```rust
state.write().await.set_dynamic_context("Relevant docs: ...");
state.write().await.append_dynamic_context("User locale: zh-CN");
state.write().await.clear_dynamic_context(); // reset when stale
```

### Clearing history

When you want to start fresh:

```rust
let mut state_lock = state.write().await;
ChatRunner::clear_history(&agent, &mut state_lock);
```

This clears both the conversation messages and the engine's internal context (KV cache for local models).

### ChatHistory query helpers

```rust
let hist = &state.read().await.chat_history;

// Find all messages containing a keyword
let results = hist.search_by_keyword("weather");

// Sniff the latest user input
if let Some(msg) = hist.last_user_message() {
    // ...
}

// Sniff the latest assistant output
if let Some(msg) = hist.last_assistant_message() {
    // ...
}
```

## Clone-friendly

`Agent` is cheap to clone – all internal fields are `Arc`-wrapped. You can build one agent and share it across hundreds of conversations:

```rust
let agent = Agent::make(config).await?.preamble("You are helpful.");
let state = AgentState::new_shared("multi-turn");
let runner = ChatRunner::default();

for _ in 0..100 {
    let agent = agent.clone(); // cheap: just bumps Arc refcounts
    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        let _ = runner.chat(&agent, &state_clone, "Hi").await;
    });
}
```

## Error handling

All public API calls return `Result<T, AmbiError>`. The enum covers:

- `EngineError` – model init/chat failures
- `AgentError` – logic errors (e.g., duplicate tool name)
- `ToolError` – tool execution timeout or failure
- `ContextError` – prompt formatting issues
- `PipelineError` – stream disconnected
- `MaxIterationsReached` – ReAct loop exceeded the iteration limit

```rust
let agent = Agent::make(config).await?.preamble("You are helpful.");
let state = AgentState::new_shared("err-demo");
let runner = ChatRunner::default();

match runner.chat(&agent, &state, "Hello").await {
    Ok(reply) => println!("{}", reply),
    Err(AmbiError::ToolError(msg)) => eprintln!("Tool failed: {}", msg),
    Err(e) => eprintln!("Something else: {}", e),
}
```
