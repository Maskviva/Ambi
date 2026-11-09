# 工具解析器：模型语法与工具执行解耦

模型可以产生文本或 Provider 特定的工具调用。Ambi 的 `ToolCallParser` 把文本形式转换为 `(tool_name, serde_json::Value)` 参数，再由工具注册表查找 Rust 实现。**解析器不等于权限控制。**

## 默认标签协议

```text
[TOOL_CALL]{"name":"get_weather","args":{"city":"Tokyo"}}[/TOOL_CALL]
```

`TagToolParser` 支持单对象、数组和常见 Markdown 代码块；JSON 解析失败时可能产生特殊的 `__format_error__`，让模型尝试修正。截断恢复是**尽力而为**，不能保证所有错误 JSON 都能恢复。OpenAI 兼容端点的原生工具调用由 Provider 适配层转成内部格式。

## 实现接口

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
        // 用自定义逻辑解析模型文本。
        let _ = text;
        vec![]
    }
}

let agent = Agent::make(engine_config).await?
    .with_tool_parser(MyParser);
```

`format_instruction` 构建工具提示词，`get_tags` 告诉格式化器标签范围，`parse` 提取实际调用。

如果自定义协议采用特殊标签，通常还要搭配 [StreamFormatter](/zh/advanced/stream-formatter)。注意 0.4.1 的 `ToolCallParser` Trait **只有** `get_tags`、`format_instruction`、`parse`，**没有** `create_stream_formatter` 方法。

工具执行与副作用策略参见 [工具](/zh/guide/tools)。
