# Stream formatter: presentation, not protocol

`StreamFormatter` processes text chunks for display. It is a replaceable presentation strategy; it does **not** change the semantics of tool authorization or model inference.

## Built-in choices

- `PassThroughFormatter`: the default, forwards the received text.
- `StandardStreamFormatter`: recognizes configured tool-call tags and think markers, and emits labelled display text such as `[Thinking]:` and `[Content]:`.

Enable the standard formatter with:

```rust
let agent = Agent::make(engine_config).await?
    .with_standard_formatting();
```

The standard implementation maintains a bounded internal buffer (default 8192 bytes). Overflow clears the formatter's buffer and logs an error. That is a local buffer guard, **not** an end-to-end proof that all streaming workloads are memory bounded.

## Write a formatter

```rust
use ambi::types::StreamFormatter;

struct Plain;

impl StreamFormatter for Plain {
    fn push(&mut self, chunk: &str) -> String {
        chunk.to_string()
    }
    fn flush(&mut self) -> String {
        String::new()
    }
}

let agent = Agent::make(engine_config).await?
    .with_stream_formatter(|| Box::new(Plain));
```

The API takes a **factory closure** so each streaming request gets a fresh formatter instance. Its two required methods are `push(&mut self, &str) -> String` and `flush(&mut self) -> String`.

Text formatting is useful for human-readable streams; do not parse it as a structured tool/audit event transport. For the incoming tool format, see [Tool Parser](/advanced/tool-parser); for output semantics, see [Streaming](/guide/streaming).
