# 架构概览

这篇文章讲 Ambi 内部是怎么工作的。用框架不一定要全懂，但想扩展的时候会很有帮助。

## Agent 和 AgentState 是分开的

这是最重要的设计决策。`Agent` 是只读蓝图。`AgentState` 是可变的对话记忆。

```
Agent（只读，所有字段 pub(crate) / Arc 包装 → 零成本克隆）
├── llm_engine (Arc<LLMEngine>)  → 模型后端 (pub(crate))
├── config (Arc<AgentConfig>)    → 系统提示词、模板、驱逐策略
├── tools_def / tool_map         → 注册的工具和定义
├── tool_parser                  → 从 LLM 输出中解析工具调用的方式
├── cached_tool_prompt           → 预渲染的工具指令字符串
├── formatter_factory            → 流式输出清理方式
└── on_evict_handler             → 消息被驱逐时的回调（接收 &AgentState）

AgentState（可变，RwLock）
├── session_id             → 唯一会话标识（KV Cache 槽位分配、分布式追踪）
├── dynamic_context        → 易变会话数据（RAG 结果、环境变量）
├── chat_history           → 纯 FIFO 队列（仅 User / Assistant / Tool）
└── extensions             → anymap2 用于自定义状态
```

注意：Agent 的字段都是 `pub(crate)` 的——外部代码通过公开 API（`chat()`、`chat_stream()` 等）与 Agent 交互，
而不是直接访问内部字段。

这样设计的好处：

- **一个 Agent 蓝图，多组对话** —— clone 只是 Arc 引用计数 +1
- **Agent 构建只做一次** —— 包括阻塞式的引擎加载
- **State 可以序列化** —— 对话可以持久化和恢复
- **最大化 KV Cache 命中率** —— 系统提示词（静态）永远不会从头部被驱逐

## ReAct 循环

调用 `runner.chat()` 或 `runner.chat_stream()` 时，内部流程：

```
用户输入
    │
    ▼
1. 把用户消息推进 ChatHistory
2. 构建 LLMRequest
   ├─ system_prompt + dynamic_context
   ├─ cached_tool_prompt
   ├─ 过滤后的历史（仅 User/Assistant/Tool）
   ├─ 渲染好的 prompt 字符串
   └─ 提取的图片
    │
    ▼
3. LLMEngine.chat() / chat_stream()
   └─ 返回原始文本
    │
    ▼
4. ToolCallParser.parse(output)
   └─ 从文本中提取工具调用
    │
    ├─ 没有工具 → 返回文本
    │
    └─ 有工具 →
       5. 并行执行（.buffered(max_concurrency)），每个工具有超时
       6. 工具结果作为 Tool 消息推回 ChatHistory
       7. 驱逐检查（纯 FIFO，无 System 消息），回调 on_evict(state, msgs)
       8. 回到步骤 3（最多 max_iterations 次）
```

步骤 3-8 重复，直到没有工具调用、或者达到 `max_iterations`。

### ChatRunner 并发控制

`ChatRunner` 持有 `maximum_concurrency` 字段（默认 5，通过 `ChatRunner::default()` 创建），
可以对并行工具执行进行灵活的速率限制：

```rust
use ambi::ChatRunner;

// 默认：最多 5 个并发工具执行
let runner = ChatRunner::default();

// 自定义限制
let runner = ChatRunner::new(3);
```

## 模板渲染

`ChatTemplate` 定义了消息序列化为 prompt 字符串的方式。每个变体存储不同角色（system/user/assistant/tool）的前后缀标签。

```
举例：ChatML 格式
<|im_start|>system
你是一个助手。
<|im_end|>
<|im_start|>user
你好
<|im_end|>
<|im_start|>assistant
你好呀
<|im_end|>
<|im_start|>assistant   ← 从这里开始生成
```

引擎收到渲染好的 prompt 字符串。OpenAI 引擎额外收到结构化的 `LLMRequest`（system/history/tools 分开）。

## Pipeline trait

`Pipeline` 是定义执行契约的 trait。`ChatRunner` 是内置实现，你可以写自己的：

```rust
// 原生平台 (Send + Sync)
pub trait Pipeline: Send + Sync {
    fn execute(
        &self, agent: &Agent, state: &Arc<RwLock<AgentState>>, input: Vec<ContentPart>
    ) -> impl Future<Output = Result<String>> + Send;

    fn execute_stream(
        &self, agent: &Agent, state: &Arc<RwLock<AgentState>>, input: Vec<ContentPart>
    ) -> impl Future<Output = Result<Pin<Box<ReceiverStream<Result<String>>>>>> + Send;
}

// WASM (无 Send + Sync 约束)
#[cfg(target_arch = "wasm32")]
pub trait Pipeline {
    // 方法签名相同，但不含 Send + Sync
}
```

两种模式：
- **同步** —— 等全部回复准备好再返回（内部跑同样的 ReAct 循环）
- **流式** —— 返回一个 `ReceiverStream`，调用方逐条迭代

## 可扩展点（全部基于 trait）

| 可以替换什么 | Trait | 默认 |
|-------------|-------|------|
| LLM 后端 | `LLMEngineTrait` | OpenAI / llama.cpp |
| 工具实现 | `Tool` | 用户提供 |
| 工具调用解析 | `ToolCallParser` | 基于 `[TOOL_CALL]` 标签 |
| 流式格式化 | `StreamFormatter` | 透传 |
| 执行管道 | `Pipeline` | `ChatRunner` |
| 分词器 | `TokenizerTrait` | `cl100k_base` (tiktoken) |

## 跨平台运行时

`runtime` 模块抽象了平台差异：

| 函数 | 原生 (tokio) | WASM |
|------|-------------|------|
| `spawn` | `tokio::spawn` | `wasm_bindgen_futures::spawn_local` |
| `spawn_blocking` | `tokio::task::spawn_blocking` | 直接执行（单线程） |
| `sleep` | `tokio::time::sleep` | `gloo_timers::future::sleep` |
| `timeout` | `tokio::time::timeout` | Future 竞速 |
| `SendSync` | `Send + Sync` | 空 trait（无操作） |

WASM 下编译时会阻止 `llama-cpp` 特性：

```rust
#[cfg(all(target_arch = "wasm32", feature = "llama-cpp"))]
compile_error!("llama-cpp not supported on wasm32");
```
