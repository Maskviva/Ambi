# 基础 Agent

跑起来一个 Agent 之后（见[快速开始](/zh/guide/getting-started)），这篇讲日常最常用的几个模式。

## 系统提示词

系统提示词设定 Agent 的行为，每次请求都会拼在 prompt 的最前面。

```rust
let agent = Agent::make(config).await?
    .preamble("你是一个用俳句回答问题的助手。");
```

提示词长度没有硬性限制，但注意它会占 token 预算。

## 聊天模板

不同模型用不同的 prompt 格式。Ambi 内置了 9 种模板：

| 模板 | 适用模型 |
|------|----------|
| `Chatml` | OpenAI、Qwen、很多微调模型 |
| `Llama3` | Llama 3 / 3.1 系列 |
| `Deepseek` | DeepSeek V2/V3、R1 |
| `Qwen` | Qwen 2 / 2.5 |
| `Gemma` | Google Gemma |
| `Phi3` | Microsoft Phi-3 / Phi-4 |
| `Mistral` | Mistral / Mixtral |
| `Zephyr` | HuggingFace Zephyr |
| `Llama2` | 旧的 Llama 2 |

```rust
use ambi::ChatTemplateType;

let agent = Agent::make(config).await?
    .template(ChatTemplateType::Deepseek);
```

默认是 `Chatml`。如果模型需要自定义格式，可以手动构造 `ChatTemplate` 结构体。

## 多轮对话

Agent 把历史存在 `AgentState` 里。每次 `chat()` 都会追加用户消息和助手回复，下一轮自动带上上下文。

```rust
let state = Arc::new(RwLock::new(AgentState::new()));

runner.chat(&agent, &state, "我叫小明。").await?;
// 下一轮它记得"小明"
runner.chat(&agent, &state, "我叫什么名字？").await?;
// -> "你叫小明"
```

### 清空历史

想重新开始的时候：

```rust
let mut state_lock = state.write().await;
ChatRunner::clear_history(&agent, &mut state_lock);
```

这会同时清空对话消息和引擎内部上下文（本地模型会清 KV Cache）。

## 克隆友好

`Agent` 的所有内部字段都用 `Arc` 包装，克隆只是引用计数 +1。可以一个 Agent 蓝图跑几百个对话：

```rust
let agent = Agent::make(config).await?;
for _ in 0..100 {
    let agent = agent.clone();
    tokio::spawn(async move {
        runner.chat(&agent, &state, "你好").await;
    });
}
```

## 错误处理

所有公开 API 都返回 `Result<T, AmbiError>`。错误枚举包括：

- `EngineError` —— 模型初始化或推理失败
- `AgentError` —— 逻辑错误（如工具名重复）
- `ToolError` —— 工具执行超时或失败
- `ContextError` —— prompt 格式化问题
- `PipelineError` —— 流中断
- `MaxIterationsReached` —— ReAct 循环超限

```rust
match runner.chat(&agent, &state, "你好").await {
    Ok(reply) => println!("{}", reply),
    Err(AmbiError::ToolError(msg)) => eprintln!("工具执行失败：{}", msg),
    Err(e) => eprintln!("其他错误：{}", e),
}
```
