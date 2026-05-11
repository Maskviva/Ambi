# Node.js Binding

The Node.js binding lets you use Ambi from JavaScript/TypeScript with full access to OpenAI-compatible APIs, custom LLM engines, tool calling, and streaming.

## Installation

The binding is published as an npm package. It ships with pre-built native binaries for Windows x64, Linux x64, and Linux arm64.

```bash
npm install @maskviva/ambi-node
```

> **Local development:** the package lives in `bindings/node/`. Build it with `npm run build` (requires Rust toolchain).

## Quick Start

```javascript
const {Agent, AgentState, Pipeline, LLMEngineConfig, JsChatTemplateType} = require('@maskviva/ambi-node')

// 1. Configure the engine
const config = LLMEngineConfig.openai({
    apiKey: process.env.OPENAI_API_KEY,
    baseUrl: 'https://api.openai.com/v1',
    modelName: 'gpt-4o-mini',
    temp: 0.7,
    topP: 0.9,
})

// 2. Create the agent
const agent = (await Agent.make(config))
    .template(JsChatTemplateType.Chatml)
    .preamble('You are a helpful assistant.')

// 3. Chat
const state = new AgentState('session-1')
const runner = Pipeline.chatRunner(5)
const reply = await runner.chat(agent, state, 'Hello!')
```

## API Reference

| JS API | Rust Equivalent | Description |
|--------|----------------|-------------|
| `Agent.make(config)` | `Agent::make()` | Create an agent (async) |
| `agent.preamble(text)` | `agent.preamble()` | Set system prompt |
| `agent.template(type)` | `agent.template()` | Chat template type |
| `agent.customTemplate(obj)` | `agent.template(ChatTemplate)` | Custom template |
| `agent.tool(tool(...))` | `agent.tool()` | Register a tool |
| `agent.withStandardFormatting()` | `agent.with_standard_formatting()` | Enable standard formatting |
| `agent.withEvictionStrategy(...)` | `agent.with_eviction_strategy()` | Memory eviction |
| `agent.maxIterations(n)` | `agent.max_iterations()` | Max tool iterations |
| `agent.withToolTags(s, e)` | `agent.with_tool_tags()` | Custom tool tags |
| `agent.countTokens(text)` | `engine.count_tokens()` | Token counting |
| `agent.onEvict(cb)` | `agent.on_evict()` | Eviction callback |
| `AgentState(id)` | `AgentState::new()` | Session state |
| `LLMEngineConfig.openai(opts)` | `LLMEngineConfig::OpenAI()` | OpenAI engine |
| `LLMEngineConfig.custom(handler)` | `LLMEngineConfig::Custom()` | Custom JS engine |
| `Pipeline.chatRunner(n)` | `ChatRunner::new()` | Chat runner |
| `Pipeline.custom(handler)` | — | Custom JS pipeline |
| `runner.chat(...)` | `runner.chat()` | Sync chat |
| `runner.chatStream(...)` | `runner.chat_stream()` | Streaming chat |
| `stream.nextChunk()` | — | Read next token |

## Tool Registration

Tools can be registered using the lightweight `tool()` helper:

```javascript
const {tool} = require('@maskviva/ambi-node')

agent.tool(tool({
    name: 'get_weather',
    description: 'Query real-time weather for a city',
    parameters: {
        city: {type: 'string', description: 'City name'},
        unit: {type: 'string', enum: ['celsius', 'fahrenheit']},
    },
    callback: (args) => ({
        temperature: 25,
        condition: 'Sunny',
    }),
}))
```

## Custom LLM Engine

Create a custom engine from any JS function. The handler must be **synchronous** — start async work inside and call `resolveRequest()` when done:

```javascript
const {LLMEngineConfig, resolveRequest} = require('@maskviva/ambi-node')

const config = LLMEngineConfig.custom(
    (err, argsJson) => {
        const {request_id, request} = JSON.parse(argsJson)
        // Start async work
        (async () => {
            const result = await myAsyncLLMCall(request.formatted_prompt)
            resolveRequest(request_id, result)
        })()
    },
    false,  // supportsMultimodal
    null,   // streamHandler (optional)
)
```

## Streaming

```javascript
const stream = await runner.chatStream(agent, state, 'Tell me a story')
for (let chunk = await stream.nextChunk(); chunk !== null; chunk = await stream.nextChunk()) {
    process.stdout.write(chunk)
}
```
