# Ambi — Node.js Binding

<p align="center">
  <a href="https://spdx.org/licenses/Apache-2.0.html"><img src="https://img.shields.io/badge/License-Apache%202.0-blue" alt="License: Apache 2.0"></a>
  <a href="https://github.com/maskviva/ambi"><img alt="github" src="https://img.shields.io/badge/github-maskviva/Ambi-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20"></a>
  <a href="https://www.npmjs.com/package/ambi-node"><img alt="npm" src="https://img.shields.io/npm/v/ambi-node?style=for-the-badge&color=cb3837&logo=npm" height="20"></a>
</p>

Ambi is a flexible, highly customizable AI Agent framework built entirely in Rust. This package provides native Node.js bindings so you can build production-grade agents from JavaScript/TypeScript with zero compromise on performance.

- **Dual-engine architecture** — Seamlessly switch between local inference (via `llama.cpp` with GPU acceleration) and cloud APIs (OpenAI-compatible endpoints) without changing your agent code.
- **Advanced tool system** — Parallel multi-tool execution, per-tool timeouts and retries, automatic JSON Schema generation from Rust structs.
- **Intelligent context management** — Safe eviction algorithm that preserves conversation logic, preventing token overflow while keeping your agent focused.
- **Rust native, Node native** — The full Ambi framework compiled into a native `.node` addon via NAPI-RS. Zero JS overhead, full async support.

## Installation

```bash
npm install ambi-node
```

The package ships with prebuilt binaries for:
- Windows x64 & arm64
- Linux x64 & arm64 (glibc and musl)
- macOS x64 & arm64

No Rust toolchain required on the consuming machine.

## Quick Start

```js
const { Engine, Agent, AgentState, ChatRunner } = require('ambi-node');

// 1. Configure an OpenAI-compatible engine
const engine = Engine.createOpenai({
  apiKey: process.env.OPENAI_API_KEY,
  baseUrl: 'https://api.openai.com/v1',
  modelName: 'gpt-4o',
  temp: 0.7,
  topP: 0.95,
});

// 2. Build an agent
const agent = await Agent.make(engine)
  .preamble('You are a helpful assistant.')
  .setTemplate(/* ChatTemplateType.Chatml */ 0);

// 3. Create a conversation state
const state = new AgentState();

// 4. Run the chat pipeline
const runner = new ChatRunner();
const response = await runner.chat(agent, state, 'Hello!');
console.log(response);
```

## Streaming

```js
runner.chatStream(
  agent, state, 'Tell me a story.',
  (token) => process.stdout.write(token),
  ()      => console.log('\n--- done ---'),
  (err)   => console.error(err),
);
```

## Using with a Local Model

The binding exposes `Engine.createOpenai(...)`, which is compatible with any OpenAI-compatible local server (Ollama, vLLM, llama.cpp server, etc.):

```js
const engine = Engine.createOpenai({
  apiKey: 'ollama',                              // Ollama ignores the key
  baseUrl: 'http://localhost:11434/v1',
  modelName: 'llama3.2',
  temp: 0.7,
  topP: 0.95,
});
```

For direct `llama.cpp` inference (no HTTP server), you need to use the Rust crate with the `llama-cpp` feature enabled.

## API Overview

| Class | Description |
|-------|-------------|
| `Engine` | LLM inference backend — factory `createOpenai(config)` |
| `Agent` | Immutable agent blueprint — builder pattern |
| `AgentState` | Per-conversation mutable state |
| `ChatRunner` | Default ReAct execution loop — `chat` & `chatStream` |
| `Tokenizer` | Fast BPE token counter (cl100k_base) |

Full TypeScript declarations are in `index.d.ts`.

## Examples

```bash
# Chat with GPT-4o
OPENAI_API_KEY=sk-... node node_modules/ambi-node/examples/chat-cloud.js

# Streaming response
OPENAI_API_KEY=sk-... node node_modules/ambi-node/examples/chat-stream.js

# Multi-turn conversation
node node_modules/ambi-node/examples/multi-turn.js
```

All examples are also available in the [GitHub repository](https://github.com/Maskviva/Ambi/tree/main/bindings/node/examples).

## API

### Engine

```js
// Factory
Engine.createOpenai(config)

// Methods
await engine.chat(request)
engine.chatStream(request, onToken, onComplete, onError)
engine.resetContext()
engine.supportsMultimodal()
await engine.evaluateSentenceEntropy(sentence)
engine.countTokens(text)
```

### Agent

```js
// Factory
await Agent.make(engine)

// Builder chain (all return a cloned Agent)
agent.preamble(text)
agent.setTemplate(type)
agent.setCustomTemplate(template)
agent.withEvictionStrategy(strategy)
agent.withStandardFormatting()
```

### ChatRunner

```js
// Constructor
new ChatRunner()

// Methods
await runner.chat(agent, state, prompt)
runner.chatStream(agent, state, prompt, onToken, onComplete, onError)
await runner.execute(agent, state, contentParts)
runner.executeStream(agent, state, contentParts, onToken, onComplete, onError)
await ChatRunner.clearHistory(agent, state)
```

## Context Eviction

Fine-tune how aggressively old messages are evicted:

```js
const agent = await Agent.make(engine)
  .withEvictionStrategy({ maxSafeTokens: 4096 });
```

## Feature Flags (Rust crate)

When building from source, the underlying `ambi` crate supports these features:

- **`openai-api`** (default) — OpenAI-compatible cloud backend
- **`llama-cpp`** — Local inference via llama.cpp (GPU backends: `cuda`, `vulkan`, `metal`, `rocm`)
- **`macro`** — Proc-macro helpers for tool definitions

The prebuilt npm package includes all features. If you need a custom build, see the [Rust documentation](https://docs.rs/ambi).

## License

Licensed under the [Apache License, Version 2.0](https://spdx.org/licenses/Apache-2.0.html).

---

<p align="center">
  <a href="https://crates.io/crates/ambi">Rust crate</a> ·
  <a href="https://docs.rs/ambi">Rust docs</a> ·
  <a href="https://github.com/Maskviva/Ambi">GitHub</a>
</p>
