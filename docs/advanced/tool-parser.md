# Tool Parser

The tool parser extracts structured tool calls from the LLM's raw text output. This is how Ambi bridges natural language generation and function execution.

## Default: TagToolParser

The built-in parser looks for `[TOOL_CALL]` and `[/TOOL_CALL]` tags:

```
[TOOL_CALL]{"name":"get_weather","args":{"city":"Tokyo"}}[/TOOL_CALL]
```

It supports:
- **Single objects** and **arrays of objects**
- **Markdown code block wrapping** (` ```json ... ``` `)
- **Truncated JSON recovery** – if the model hits max_tokens and cuts off mid-JSON, the parser finds the last `}` and discards the trailing garbage
- **Malformed JSON** – if parsing fails entirely, it emits a `__format_error__` call that triggers a correction prompt in the next LLM turn

```rust
// From the code:
pub(crate) fn extract_and_push_call(json_str: &str, calls: &mut Vec<(String, Value)>) {
    // 1. Try complete parse
    // 2. If that fails, find last '}' and try truncated parse
    // 3. If both fail, push __format_error__
}
```

## Custom parser

Implement `ToolCallParser`:

```rust
use ambi::types::ToolCallParser;

struct JsonModeParser;

impl ToolCallParser for JsonModeParser {
    fn get_tags(&self) -> (String, String) {
        ("<tool>".into(), "</tool>".into())
    }

    fn format_instruction(&self, tools_json: &str) -> String {
        format!(
            "Available functions:\n{}\n\nCall format:\n<tool>{{"name":"fn_name","args":{{...}}}}</tool>",
            tools_json
        )
    }

    fn parse(&self, text: &str) -> Vec<(String, serde_json::Value)> {
        // Your parsing logic here
        todo!()
    }
}
```

### What each method does

| Method | Purpose |
|--------|---------|
| `get_tags()` | Returns the start/end tags – used by `StandardStreamFormatter` to hide them during streaming |
| `format_instruction()` | Generates the system prompt that tells the LLM how to format tool calls |
| `parse()` | Scans the LLM output text, extracts tool names and arguments |

### format_instruction is cached

The instruction string is computed once when tools are registered (`tool()` builder call) and cached. It's not regenerated per request. If your parser's instructions change based on runtime state, you'll need to handle that differently.

## Using a custom parser

```rust
let agent = Agent::make(config).await?
    .with_tool_parser(JsonModeParser);
```

## StreamFormatter coupling

A parser can provide its own stream formatter. The default parser doesn't override this, but if your custom format needs specific streaming cleanup, override `create_stream_formatter()`:

```rust
impl ToolCallParser for MyParser {
    fn create_stream_formatter(&self) -> Box<dyn StreamFormatter> {
        Box::new(PassThroughFormatter)
    }
    // ...
}
```

This is called during `with_standard_formatting()` to create the formatter factory.
