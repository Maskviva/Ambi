# 流式格式化器：负责展示，不负责协议

`StreamFormatter` 处理流式文本的展示方式。它是可替换的**输出层策略**，并不改变工具权限或推理执行语义。

## 内置实现

- `PassThroughFormatter`：默认实现，直接透传文本。
- `StandardStreamFormatter`：识别工具标签与思考标签，输出如 `[Thinking]:`、`[Content]:` 等展示文本。

开启标准格式化：

```rust
let agent = Agent::make(engine_config).await?
    .with_standard_formatting();
```

标准实现内部有缓冲区大小上限（默认 8192 字节）。超过上限会清空格式化器的缓冲并记录错误。这是一种**局部缓冲保护**，不代表所有流式执行过程都有端到端的内存上限。

## 自定义格式化器

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

API 接收的是**工厂闭包**，每次流式请求会创建新的 Formatter。必需方法只有 `push(&mut self, &str) -> String` 与 `flush(&mut self) -> String`。

这种格式化输出适合 UI 展示，不适合当作稳定的结构化工具审计协议。输入语法参见 [工具解析器](/zh/advanced/tool-parser)，输出语义参见 [流式响应](/zh/guide/streaming)。
