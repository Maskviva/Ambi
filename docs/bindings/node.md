# Node.js binding

Ambi's Rust core has a separate Node.js binding. **The binding has its own package version and API surface**: the `bindings/node/package.json` in this repository declares `@maskviva/ambi-node` **0.2.1**, while the Rust core is 0.4.1.

A binding exposes useful runtime capabilities; it is not necessarily feature-for-feature synchronized with every new core feature. In particular, the 0.4.1 Rust Leaf API is **not documented as a Node.js binding API** here.

## Install

```bash
npm install @maskviva/ambi-node
```

Or build the native addon from `bindings/node/` using that package's build instructions. Native binaries and packaging constraints are platform-dependent.

## Minimal example

```js
const { Agent, AgentState, Pipeline, LLMEngineConfig } = require('@maskviva/ambi-node');

async function main() {
  const cfg = LLMEngineConfig.openai({
    apiKey: process.env.OPENAI_API_KEY,
    baseUrl: 'https://api.openai.com/v1',
    modelName: 'gpt-4o-mini',
    temp: 0.7,
    topP: 0.9,
  });

  const agent = (await Agent.make(cfg)).preamble('You are a concise assistant.');
  const state = new AgentState('node-session-1');
  const runner = Pipeline.chatRunner(5);
  console.log(await runner.chat(agent, state, 'Hello, Ambi!'));
}

main().catch(console.error);
```

## Common binding APIs

| JS | Meaning |
| --- | --- |
| `LLMEngineConfig.openai(opts)` | OpenAI-compatible backend configuration |
| `Agent.make(config)` | Initialize a shared agent |
| `agent.preamble(text)` | System prompt |
| `agent.tool(...)` | Register a JS-defined tool |
| `AgentState(id)` | Session state |
| `Pipeline.chatRunner(n)` | Built-in chat strategy |
| `Pipeline.custom(...)` | JS-provided execution strategy |
| `runner.chat(...)`, `runner.chatStream(...)` | Full or streaming response |

For tool helpers, custom engines and streaming examples, prefer the versioned source in [`bindings/node/examples`](https://github.com/Maskviva/Ambi/tree/main/bindings/node/examples) and the generated `lib/index.d.ts` matching your installed package.

## Cross-language design boundary

The tree / branch / leaf metaphor describes **Rust core architecture**; it does not mean `Pipeline.chatRunner` on Node automatically exposes Rust's `with_leaf()`. Check bindings separately when adopting new native features. See [Design philosophy](/advanced/design-philosophy) and [Leaf](/guide/leaf).
