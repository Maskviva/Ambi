# 工具解析器

工具解析器从 LLM 的原始文本输出中提取结构化的工具调用。这就是 Ambi 连接自然语言生成和函数执行的方式。

## 默认：TagToolParser

内置解析器查找 `[TOOL_CALL]` 和 `[/TOOL_CALL]` 标签：

```
[TOOL_CALL]{"name":"get_weather","args":{"city":"东京"}}[/TOOL_CALL]
```

它支持：
- **单个对象**和**对象数组**
- **Markdown 代码块包裹**（` ```json ... ``` `）
- **截断 JSON 恢复** —— 模型达到 max_tokens 导致 JSON 被截断时，解析器找到最后一个 `}` 丢弃后面的垃圾
- **格式错误的 JSON** —— 如果完全解析失败，产生 `__format_error__` 调用，触发下一轮纠错提示

```rust
// 来自代码：
pub(crate) fn extract_and_push_call(json_str: &str, calls: &mut Vec<(String, Value)>) {
    // 1. 尝试完整解析
    // 2. 失败则找最后一个 '}' 尝试截断解析
    // 3. 都失败则推入 __format_error__
}
```

## 自定义解析器

实现 `ToolCallParser`：

```rust
use ambi::types::ToolCallParser;

struct JsonModeParser;

impl ToolCallParser for JsonModeParser {
    fn get_tags(&self) -> (String, String) {
        ("<tool>".into(), "</tool>".into())
    }

    fn format_instruction(&self, tools_json: &str) -> String {
        format!(
            "可用函数：\n{}\n\n调用格式：\n<tool>{{\"name\":\"函数名\",\"args\":{{...}}}}</tool>",
            tools_json
        )
    }

    fn parse(&self, text: &str) -> Vec<(String, serde_json::Value)> {
        // 你的解析逻辑
        todo!()
    }
}
```

### 每个方法的作用

| 方法 | 用途 |
|------|------|
| `get_tags()` | 返回开始/结束标签——`StandardStreamFormatter` 用它来隐藏流中的原始标签 |
| `format_instruction()` | 生成告诉 LLM 如何格式化工具调用的系统提示 |
| `parse()` | 扫描 LLM 输出文本，提取工具名和参数 |

### format_instruction 是缓存的

指令字符串在注册工具时（`tool()` builder 调用）计算一次并缓存。不会每次请求重新生成。如果你的解析器指令依赖运行时状态，需要另做处理。

## 使用自定义解析器

```rust
let agent = Agent::make(config).await?
    .with_tool_parser(JsonModeParser);
```

## 与流式格式化器的配合

解析器可以提供自己的流式格式化器。默认解析器不覆盖这个，但如果你的自定义格式需要特定的流式清理，可以覆盖 `create_stream_formatter()`：

```rust
impl ToolCallParser for MyParser {
    fn create_stream_formatter(&self) -> Box<dyn StreamFormatter> {
        Box::new(PassThroughFormatter)
    }
    // ...
}
```

这在 `with_standard_formatting()` 创建格式化器工厂时会被调用。
