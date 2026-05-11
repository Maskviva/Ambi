# Node.js 绑定

Node.js 绑定让你在 JavaScript/TypeScript 中使用 Ambi，支持 OpenAI 兼容 API、自定义 LLM 引擎、工具调用和流式输出。

## 安装

绑定已发布为 npm 包，内置 Windows x64、Linux x64 和 Linux arm64 预编译原生二进制。

```bash
npm install @maskviva/ambi-node
```

> **本地开发：** 代码在 `bindings/node/` 目录下。使用 `npm run build` 构建（需要 Rust 工具链）。

## 快速开始

```javascript
const {Agent, AgentState, Pipeline, LLMEngineConfig, JsChatTemplateType} = require('@maskviva/ambi-node')

// 1. 配置引擎
const config = LLMEngineConfig.openai({
    apiKey: process.env.OPENAI_API_KEY,
    baseUrl: 'https://api.openai.com/v1',
    modelName: 'gpt-4o-mini',
    temp: 0.7,
    topP: 0.9,
})

// 2. 创建 Agent
const agent = (await Agent.make(config))
    .template(JsChatTemplateType.Chatml)
    .preamble('You are a helpful assistant.')

// 3. 开始对话
const state = new AgentState('session-1')
const runner = Pipeline.chatRunner(5)
const reply = await runner.chat(agent, state, 'Hello!')
```

## API 参考

| JS API | Rust 对应 | 说明 |
|--------|----------|------|
| `Agent.make(config)` | `Agent::make()` | 创建 Agent（异步） |
| `agent.preamble(text)` | `agent.preamble()` | 设置系统提示词 |
| `agent.template(type)` | `agent.template()` | 聊天模板类型 |
| `agent.customTemplate(obj)` | `agent.template(ChatTemplate)` | 自定义模板 |
| `agent.tool(tool(...))` | `agent.tool()` | 注册工具 |
| `agent.withStandardFormatting()` | `agent.with_standard_formatting()` | 启用标准格式化 |
| `agent.withEvictionStrategy(...)` | `agent.with_eviction_strategy()` | 上下文驱逐策略 |
| `agent.maxIterations(n)` | `agent.max_iterations()` | 最大工具迭代次数 |
| `agent.withToolTags(s, e)` | `agent.with_tool_tags()` | 自定义工具标签 |
| `agent.countTokens(text)` | `engine.count_tokens()` | Token 计数 |
| `agent.onEvict(cb)` | `agent.on_evict()` | 驱逐回调 |
| `AgentState(id)` | `AgentState::new()` | 会话状态 |
| `LLMEngineConfig.openai(opts)` | `LLMEngineConfig::OpenAI()` | OpenAI 引擎 |
| `LLMEngineConfig.custom(handler)` | `LLMEngineConfig::Custom()` | 自定义 JS 引擎 |
| `Pipeline.chatRunner(n)` | `ChatRunner::new()` | 对话运行器 |
| `Pipeline.custom(handler)` | — | 自定义 JS 管道 |
| `runner.chat(...)` | `runner.chat()` | 同步对话 |
| `runner.chatStream(...)` | `runner.chat_stream()` | 流式对话 |
| `stream.nextChunk()` | — | 读取下一个 token |

## 工具注册

使用轻量的 `tool()` 辅助函数注册工具：

```javascript
const {tool} = require('@maskviva/ambi-node')

agent.tool(tool({
    name: 'get_weather',
    description: '查询指定城市的实时天气',
    parameters: {
        city: {type: 'string', description: '城市名称'},
        unit: {type: 'string', enum: ['celsius', 'fahrenheit']},
    },
    callback: (args) => ({
        temperature: 25,
        condition: 'Sunny',
    }),
}))
```

`tool()` 辅助函数自动从 JS 对象推导 JSON Schema，将回调包装成 Rust 侧的 `Tool` trait 实现。

## 自定义 LLM 引擎

从任意 JS 函数创建自定义引擎。处理函数必须是**同步的**——在内部启动异步工作，完成后调用 `resolveRequest()`：

```javascript
const {LLMEngineConfig, resolveRequest} = require('@maskviva/ambi-node')

const config = LLMEngineConfig.custom(
    (err, argsJson) => {
        const {request_id, request} = JSON.parse(argsJson)
        // 启动异步工作
        (async () => {
            const result = await myAsyncLLMCall(request.formatted_prompt)
            resolveRequest(request_id, result)
        })()
    },
    false,  // supportsMultimodal
    null,   // streamHandler（可选）
)
```

## 流式响应

```javascript
const stream = await runner.chatStream(agent, state, '讲个故事')
for (let chunk = await stream.nextChunk(); chunk !== null; chunk = await stream.nextChunk()) {
    process.stdout.write(chunk)
}
```

## 模板字符串

内置模板以函数形式提供：

```javascript
const {chatmlTemplate, deepseekTemplate} = require('@maskviva/ambi-node')
const tpl = deepseekTemplate()
console.log(tpl.systemPrefix) // <|SYS_START|>\n
```

可用：`chatmlTemplate`, `llama3Template`, `gemmaTemplate`, `phi3Template`, `zephyrTemplate`, `deepseekTemplate`, `qwenTemplate`, `mistralTemplate`, `llama2Template`。
