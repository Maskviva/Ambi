# Stream Formatter

A stream formatter processes the LLM's output token-by-token in real time, deciding what to show the user and what to hide.

## Why you need one

When the LLM outputs raw text, it may include:
- `<think>` / `</think>` tags (reasoning blocks)
- `[TOOL_CALL]...[/TOOL_CALL]` blocks (raw tool call JSON)

These are useful for the machine but noisy for humans. The formatter strips or replaces them before the text reaches the user.

## Default: PassthroughFormatter

By default, nothing is filtered – all text goes straight through:

```rust
pub struct PassThroughFormatter;
impl StreamFormatter for PassThroughFormatter {
    fn push(&mut self, token: &str) -> String {
        token.to_string()
    }
    fn flush(&mut self) -> String {
        String::new()
    }
}
```

## Standard formatting

```rust
let agent = Agent::make(config).await?
    .with_standard_formatting();
```

This enables `StandardStreamFormatter`, which:

1. Scans each incoming token for the tool start/end tags and think tags
2. Buffers text within tool call blocks and suppresses it
3. Replaces think blocks with `[Thinking]:\n`
4. Labels non-tool content with `[Content]:`
5. Enforces a max buffer size (8KB by default) to prevent OOM on large chunks

Example transformation:

```
Raw LLM output:
  Let me think about this
  <think>
  The user is asking about the weather...
  </think>
  [TOOL_CALL]{"name":"get_weather","args":{"city":"Tokyo"}}[/TOOL_CALL]
  The weather in Tokyo is...

Formatted output:
  [Thinking]:
  The user is asking about the weather...
  [Content]:
  The weather in Tokyo is...
```

## Custom StreamFormatter

Implement the `StreamFormatter` trait:

```rust
use ambi::types::StreamFormatter;

struct MyFormatter;

impl StreamFormatter for MyFormatter {
    fn push(&mut self, token: &str) -> String {
        // Simple: just uppercase everything
        token.to_uppercase()
    }

    fn flush(&mut self) -> String {
        String::new()
    }
}
```

Then inject it:

```rust
let agent = Agent::make(config).await?
    .with_stream_formatter(|| Box::new(MyFormatter));
```

The `with_stream_formatter` method takes a **factory closure** (not an instance) because a new formatter is created per streaming request. This is important for stateful formatters that accumulate buffers.

## When the formatter is called

- In streaming mode: every LLM token chunk goes through `push()`, and `flush()` runs after the stream ends.
- In sync mode: the full output is passed through a formatter once, but the pipeline constructs it internally from `push()` + `flush()` calls.

## Buffer overflow protection

`StandardStreamFormatter` has a hard cap (`max_buffer_size`, default 8192 bytes). If the buffer exceeds it, the formatter clears itself and logs an error. This is a safety net against pathological LLM output.
