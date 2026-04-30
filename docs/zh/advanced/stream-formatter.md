# 流式格式化器

流式格式化器逐 token 处理 LLM 的输出，决定哪些展示给用户，哪些隐藏。

## 为什么需要它

LLM 输出的原始文本里可能包含：
- `<think>` / `</think>` 标签（推理过程）
- `[TOOL_CALL]...[/TOOL_CALL]` 块（工具调用的原始 JSON）

这些对机器有用，但对人来说太吵了。格式化器在处理过程中把它们替换或隐藏掉。

## 默认：透传

默认情况下什么都不会过滤——所有文本原样输出。

## 标准格式化

```rust
let agent = Agent::make(config).await?
    .with_standard_formatting();
```

这会启用 `StandardStreamFormatter`，它：

1. 扫描每个进来的 token，查找工具标签和 think 标签
2. 缓存工具调用块内的文本，不展示
3. 将 think 块替换成 `[Thinking]:\n`
4. 给非工具内容加 `[Content]:` 标签
5. 有最大缓冲区上限（默认 8KB），防止 OOM

转换示例：

```
LLM 原始输出：
  让我想想
  <think>
  用户在问天气...
  </think>
  [TOOL_CALL]{"name":"get_weather","args":{"city":"东京"}}[/TOOL_CALL]
  东京的天气是...

格式化后：
  [Thinking]:
  用户在问天气...
  [Content]:
  东京的天气是...
```

## 自定义格式化器

实现 `StreamFormatter` trait：

```rust
use ambi::types::StreamFormatter;

struct MyFormatter;

impl StreamFormatter for MyFormatter {
    fn push(&mut self, token: &str) -> String {
        token.to_uppercase() // 简单：全大写
    }

    fn flush(&mut self) -> String {
        String::new()
    }
}
```

注入：

```rust
let agent = Agent::make(config).await?
    .with_stream_formatter(|| Box::new(MyFormatter));
```

`with_stream_formatter` 接受一个**工厂闭包**（不是实例），因为每次流式请求都会新建一个格式化器。对于有状态（需要积累缓冲区）的格式化器，这很重要。

## 什么时候调用

- 流式模式：每个 LLM token 块都经过 `push()`，流结束后调 `flush()`
- 同步模式：整段输出通过格式化器过一次

## 缓冲区溢出保护

`StandardStreamFormatter` 有硬上限（`max_buffer_size`，默认 8192 字节）。缓冲区超限时会清空自己并打错误日志。这是对异常 LLM 输出的安全网。
