# 流式输出与取消边界

Pipeline 决定如何输出结果。内置 `ChatRunner` 提供返回完整文本的 `chat()` 与增量输出的 `chat_stream()`；二者底层使用同一套 ReAct 执行循环。

## 消费文本流

```rust
use futures::StreamExt;

let mut stream = runner.chat_stream(&agent, &state, "解释项目设计").await?;
while let Some(item) = stream.next().await {
    match item {
        Ok(text) => print!("{text}"),
        Err(err) => eprintln!("stream error: {err}"),
    }
}
```

需要额外依赖 `futures = "0.3"` 才能使用 `StreamExt`。返回的是 `Result<String>` 项构成的**文本流**，不是结构化模型／工具事件协议。应用可将文本转发至 SSE 或 WebSocket。

## 模型文本与工具展示

`ChatRunner` 流式输出可能包含工具相关的展示文本，具体取决于 `StreamFormatter`。可以使用 `agent.with_standard_formatting()`，或者通过 `with_stream_formatter(...)` 指定自定义格式化器。

若业务需要机器可解析的工具审计事件，不应该把展示文本当成稳定协议。参见 [流式格式化器](/zh/advanced/stream-formatter)。

## 取消不等于事务回滚

接收端被丢弃后，Runner 可以发现客户端已不再消费响应，继而停止不必要的工作。OpenAI 流式路径与本地 llama.cpp 也有取消感知机制。但不能把这种机制理解成跨系统的事务保证：

- 远程 Provider 可能已经接收请求。
- 工具 Future 被取消时，外部副作用可能仍继续。
- 本地推理 Worker 有自己的取消检测点。

应由业务层处理关键外部操作。流式父级调用 Leaf 时，Leaf 内部执行的是**同步循环**，最后只向父级返回一个工具结果，不转发自身的 Token 流。

## 同步与流式

| API | 输出 | 适用场景 |
| --- | --- | --- |
| `runner.chat(...)` | `Result<String>` | 后台作业、批处理、简单应用 |
| `runner.chat_stream(...)` | `Result<String>` 文本流 | 交互式输出 |

继续：[工具](/zh/guide/tools) · [Leaf](/zh/guide/leaf) · [架构](/zh/advanced/architecture)。
