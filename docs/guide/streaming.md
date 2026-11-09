# Streaming and cancellation boundaries

A pipeline decides what it emits. The built-in `ChatRunner` offers full-result `chat()` and incremental `chat_stream()` using the same underlying ReAct loop.

## Consume a stream

```rust
use futures::StreamExt;

let mut stream = runner.chat_stream(&agent, &state, "Explain the design").await?;
while let Some(item) = stream.next().await {
    match item {
        Ok(text) => print!("{text}"),
        Err(err) => eprintln!("stream error: {err}"),
    }
}
```

Add `futures = "0.3"` for `StreamExt`. The return value is a stream of `Result<String>` items; these are **text chunks**, not structured model/tool events. Applications can forward them through SSE or WebSocket.

## Model text vs. tool activity

A ChatRunner stream may include text and tool-related display output, depending on the configured `StreamFormatter`. Use `agent.with_standard_formatting()` for standard cleanup or provide a custom formatter via `with_stream_formatter(...)`.

For machine-readable tool traces or audit events, do not assume the stream's display text is a stable protocol. See [Stream Formatter](/advanced/stream-formatter).

## Cancellation is best-effort across boundaries

Dropping the receiver informs the runner that the client is no longer listening, allowing it to stop useful work. OpenAI streaming and local llama.cpp also have cancellation-aware paths. However, cancellation is not a universal transactional guarantee:

- A remote provider may already have received a request.
- A tool future may be dropped while its external side effect continues.
- A queued or ongoing local inference command has its own worker cancellation checkpoints.

Design business actions accordingly. For a Leaf called from a streaming parent, Leaf's internal loop is synchronous and returns a **single final tool result**, not its inner token stream.

## Sync vs. stream

| API | Output | Typical use |
| --- | --- | --- |
| `runner.chat(...)` | `Result<String>` | Jobs, evaluations, simple applications |
| `runner.chat_stream(...)` | Stream of `Result<String>` | Interactive response display |

More: [Tools](/guide/tools) · [Leaf](/guide/leaf) · [Architecture](/advanced/architecture).
