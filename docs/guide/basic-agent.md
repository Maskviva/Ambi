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
let state = Arc::new(RwLock::new(AgentState::new()));

runner.chat(&agent, &state, "My name is Alice.").await?;
// The assistant remembers "Alice" in the next turn:
runner.chat(&agent, &state, "What's my name?").await?;
// -> "Alice"
```

### Clearing history

When you want to start fresh:

```rust
let mut state_lock = state.write().await;
ChatRunner::clear_history(&agent, &mut state_lock);
```

This clears both the conversation messages and the engine's internal context (KV cache for local models).

## Clone-friendly

`Agent` is cheap to clone – all internal fields are `Arc`-wrapped. You can build one agent and share it across hundreds of conversations:

```rust
let agent = Agent::make(config).await?;
for _ in 0..100 {
    let agent = agent.clone(); // cheap: just bumps Arc refcounts
    tokio::spawn(async move {
        runner.chat(&agent, &state, "Hi").await;
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
match runner.chat(&agent, &state, "Hello").await {
    Ok(reply) => println!("{}", reply),
    Err(AmbiError::ToolError(msg)) => eprintln!("Tool failed: {}", msg),
    Err(e) => eprintln!("Something else: {}", e),
}
```
