# Streaming

Chat streaming lets you display tokens as they are generated, rather than waiting for the full response.

## Basic usage

```rust
use futures::StreamExt;

let mut stream = runner.chat_stream(&agent, &state, "Tell me a story").await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(text) => print!("{}", text),
        Err(e) => eprintln!("Stream error: {}", e),
    }
}
```

The stream yields `Result<String>` chunks. In a terminal you'd print each chunk; in a web server you'd send them as SSE or WebSocket frames.

## How streaming interacts with tools

When the agent is in streaming mode and a tool call happens, the tool result blocks are also pushed into the stream as formatted strings. Your client sees something like:

```
[Thinking]:
The user asked about the weather...
[Content]:
Let me check.
[TOOL_CALL]: get_weather({"city":"Tokyo"})
```

With `with_standard_formatting()`, these tool call labels are cleaned up automatically (see [Stream Formatter](/advanced/stream-formatter)).

## Client disconnect handling

If the client drops the connection (stream receiver is dropped), the framework detects this via the mpsc channel and:
1. Stops consuming LLM tokens
2. Cancels any pending tool executions ("ghost cancellation")
3. Logs a warning and stops

This prevents wasted inference and tool execution on abandoned requests.

## Sync vs streaming

| Mode | Call | Returns | Use case |
|------|------|---------|----------|
| Sync | `runner.chat()` | `Result<String>` | Simple requests, batch processing |
| Stream | `runner.chat_stream()` | `ReceiverStream<Result<String>>` | Chat UIs, real-time displays |

Both modes run the same ReAct loop internally. The difference is only in how the output is delivered.
