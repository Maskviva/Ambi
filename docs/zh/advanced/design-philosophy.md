# 设计理念

Ambi 的设计理念贯穿于整个框架的演进脉络。理解这四条原则，能帮你在扩展框架时做出更好的决策。

---

## 1. 把自由还给开发者——一切皆可替换

Ambi 拒绝"约定优于配置"的隐形绑定。**LLM 后端、工具解析器、流式格式化器、甚至整个执行管道，全部通过 trait 暴露为可替换的组件。**

| 可替换组件 | Trait | 替换方式 |
|-----------|-------|---------|
| LLM 后端 | `LLMEngineTrait` | `LLMEngineConfig::Custom(...)` |
| 工具调用解析器 | `ToolCallParser` | `agent.with_tool_parser(MyParser)` |
| 流式格式化器 | `StreamFormatter` | `agent.with_stream_formatter(|| Box::new(MyFormatter))` |
| 执行管道 | `Pipeline` | `Pipeline.custom(handler)` 或实现 `Pipeline` |
| 分词器 | `TokenizerTrait` | `engine.with_custom_tokenizer(MyTokenizer)` |

在本地 `llama.cpp` 和云端 OpenAI 之间一行配置切换，也可以实现 `LLMEngineTrait` 接入任意私有模型。这种 **trait-first** 的设计让框架成为一组可组装的"轮子"，而不是一辆封死引擎盖的"整车"。

```rust
// 一行切换引擎：
let config = LLMEngineConfig::OpenAI(openai_cfg);
//  vs
let config = LLMEngineConfig::Llama(llama_cfg);
```

## 2. 极致的开发体验——五行使者

从第一天起，Ambi 就追求**用最少的代码创造生产级 Agent**。5 行核心代码即可启动一个带云端推理的对话代理：

```rust
let agent = Agent::make(config).await?
    .preamble("You are a helpful assistant.");
let state = AgentState::new_shared("session-1");
let runner = ChatRunner::default();
let reply = runner.chat(&agent, &state, "Hello!").await?;
```

`#[tool]` 过程宏进一步将工具定义缩减为普通异步 Rust 函数，自动生成 JSON Schema、超时和重试逻辑：

```rust
#[tool(name = "get_weather", description = "Get weather for a city")]
async fn get_weather(city: String) -> Result<WeatherOutput, ToolErr> {
    // 你的逻辑——无需手动实现 trait，无需手写 schema
}
```

这不是魔法，而是通过精心设计的零成本抽象和智能默认值，把繁琐的样板代码压缩进框架底层。

## 3. 坚不可摧的鲁棒性——生产环境不是游乐场

Ambi 在设计上对藏匿在长尾中的失败模式有近乎偏执的防范：

### OOM 保护
流式缓冲区有硬上限（`max_buffer_size`，默认 8192）。缓冲区超限时自动清空并记录错误，防止失控生成撑爆内存。

### 智能上下文驱逐
从早期基于字符串长度的估算，演进到依托精确 token 累加器的 O(1) FIFO 算法：

```
每条消息 → (Arc<Message>, 精确 token 数)
total_tokens: 缓存累加和 → O(1) 查询
驱逐策略: 纯 FIFO，不驱逐 System 消息
```

`User` 消息作为安全切割点——历史记录仅存储 `User`、`Assistant` 和 `Tool` 消息（System 完全解耦），在防 token 溢出的同时维持对话逻辑连贯。

### 工具执行安全网
- **非幂等工具绝不重试**（杜绝重复支付）
- **幂等工具**按可配置次数重试
- **客户端断连立即中止"幽灵工具"执行**——通过 `tokio::select!` 内建于框架
- 框架层保证了**取消安全性（cancel safety）**

```rust
// 来自 tool_handler.rs 的核心安全模式：
tokio::select! {
    res = run_future => { /* 工具完成 */ }
    _ = async { tx_clone.closed().await } => {
        // 客户端断连，中止幽灵工具
    }
}
```

### 推理状态完整性
本地引擎（`llama.cpp`）通过 `snapshot/restore` 机制，在任何解码失败或流中断后，都能将 KV Cache 和会话状态恢复到一致点，彻底消除隐蔽的状态污染。

```rust
// 来自 session.rs：
pub fn snapshot(&self) -> (Vec<LlamaToken>, Vec<u8>, i32);
pub fn restore(&mut self, snapshot: ...);
```

## 4. 一次编写，随处运行——原生与 WASM 统一心智模型

Ambi 内置了跨平台的异步运行时抽象（`runtime.rs`），将 Tokio 原生的 `spawn_blocking` 等能力与 WASM 的 `wasm-bindgen-futures` 无感适配：

| 函数 | 原生 (Tokio) | WASM |
|------|-------------|------|
| `spawn` | `tokio::spawn` | `wasm_bindgen_futures::spawn_local` |
| `spawn_blocking` | `tokio::task::spawn_blocking` | 直接执行（单线程） |
| `sleep` | `tokio::time::sleep` | `gloo_timers::future::sleep` |
| `timeout` | `tokio::time::timeout` | Future 竞速 |
| `Send + Sync` | 强制要求 | 放宽（空标记） |

你在服务器上编写的 Agent 代码，几乎不用修改即可编译到浏览器里运行。硬件相关特性（`llama.cpp`）会被编译期安全剔除：

```rust
#[cfg(all(target_arch = "wasm32", feature = "llama-cpp"))]
compile_error!("llama-cpp 特性在 wasm32 上不受支持");
```

同一套 API，同一组 trait，在云端、边缘和设备端之间无缝迁移。

---

## 总结

| 原则 | 核心思想 | 代码佐证 |
|------|---------|---------|
| 高自由 | 一切皆 trait | `LLMEngineTrait`, `Pipeline`, `ToolCallParser`, `StreamFormatter` |
| 开发体验 | 五行使者 | `#[tool]` 宏，fluent `Agent::make().preamble().template()` |
| 鲁棒性 | 生产级安全网 | FIFO 驱逐、幽灵取消、snapshot/restore、OOM 防护 |
| 可移植性 | 原生 ↔ WASM | `runtime.rs` polyfill，编译期特性门控 |
