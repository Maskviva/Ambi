# Tool parser: separate model syntax from tool execution

Models emit text or protocol-specific tool calls. Ambi's `ToolCallParser` translates the text form into `(tool_name, serde_json::Value)` arguments; the tool registry then decides which Rust operation runs. **A parser is not a permission check.**

## Default tag-based protocol

```text
[TOOL_CALL]{"name":"get_weather","args":{"city":"Tokyo"}}[/TOOL_CALL]
```

`TagToolParser` handles single objects, arrays and common Markdown code fences. If it cannot parse JSON, it can emit the special `__format_error__` call to request model correction. Its best-effort truncation recovery is **not a guarantee** of recovering malformed output. OpenAI-compatible native tool calls are translated into the internal format by the provider layer.

## Implement the interface

```rust
use ambi::types::ToolCallParser;
use serde_json::Value;

struct MyParser;

impl ToolCallParser for MyParser {
    fn get_tags(&self) -> (String, String) {
        ("<tool>".into(), "</tool>".into())
    }

    fn format_instruction(&self, tools_json: &str) -> String {
        format!("Available tools: {tools_json}. Use <tool>JSON</tool>.")
    }

    fn parse(&self, text: &str) -> Vec<(String, Value)> {
        // Replace with your own format parser.
        let _ = text;
        vec![]
    }
}

let agent = Agent::make(engine_config).await?
    .with_tool_parser(MyParser);
```

`format_instruction` is used when building tool prompts; `get_tags` lets standard formatting know which tool markers to handle; `parse` supplies decoded calls.

If your custom parser uses unusual markers, consider also injecting a custom [`StreamFormatter`](/advanced/stream-formatter). The `ToolCallParser` trait itself defines only `get_tags`, `format_instruction`, and `parse` in 0.4.1—**not** `create_stream_formatter`.

See [Tools](/guide/tools) for execution policy.
