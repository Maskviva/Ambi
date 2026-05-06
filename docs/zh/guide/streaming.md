# 流式响应

流式输出让你边生成边展示，不用等全部回复。

## 基本用法

```rust
use futures::StreamExt;

let mut stream = runner.chat_stream(&agent, &state, "讲个故事").await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(text) => print!("{}", text),
        Err(e) => eprintln!("流错误：{}", e),
    }
}
```

每次 `yield` 的是 `Result<String>`。终端里直接打印就行了；Web 服务里可以转成 SSE 或 WebSocket 帧发到前端。

## WASM 浏览器流式传输

Ambi 的流式 API 在 WASM 目标下原生支持浏览器环境。OpenAI 提供者使用浏览器原生的 `fetch` 和 `ReadableStream` API——
不需要额外的 polyfill。同样的 `chat_stream()` 代码在原生和浏览器下都适用。

参考 [`examples/webAssembly`](https://github.com/maskviva/ambi/tree/main/examples/webAssembly)
查看包含 UI 切换演示的在线浏览器示例。

## 流模式与工具的交互

流模式下，工具调用的结果块也会被推到流里。你看到的可能是：

```
[Thinking]:
用户问天气...
[Content]:
让我查一下。
[TOOL_CALL]: get_weather({"city":"东京"})
```

如果启用了 `with_standard_formatting()`，这些 tool 标签会被自动清理。

## 客户端断连处理

客户端断开连接（stream receiver 被 drop）时，框架会：
1. 停止消费 LLM token
2. 取消正在执行的工具（"幽灵调用取消"）
3. 记录日志并停止

这避免了已放弃请求上的浪费推理和工具执行。

## 同步 vs 流式

| 模式 | 调用 | 返回值 | 适用场景 |
|------|------|--------|---------|
| 同步 | `runner.chat()` | `Result<String>` | 简单请求、批量处理 |
| 流式 | `runner.chat_stream()` | `ReceiverStream<Result<String>>` | 聊天界面、实时展示 |

两种模式内部跑的是同一个 ReAct 循环，区别只在输出交付方式。
