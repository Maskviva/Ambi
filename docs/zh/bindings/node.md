# Node.js 绑定

Ambi 的 Rust Core 有独立的 Node.js 绑定。**绑定包拥有独立版本与 API 范围**：本仓库 `bindings/node/package.json` 声明 `@maskviva/ambi-node` **0.2.1**，而 Rust Core 为 0.4.1。

绑定提供对 Runtime 常用能力的访问，但不代表所有新增 Rust 功能都同时暴露。特别是，本文**不承诺** 0.4.1 的 Rust Leaf API 已经出现在 Node.js 绑定中。

## 安装

```bash
npm install @maskviva/ambi-node
```

也可以根据 `bindings/node/` 的构建说明自行编译 Native Addon。预编译二进制和平台支持取决于具体安装包。

## 最小示例

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

  const agent = (await Agent.make(cfg)).preamble('你是一个简洁的助手。');
  const state = new AgentState('node-session-1');
  const runner = Pipeline.chatRunner(5);
  console.log(await runner.chat(agent, state, '你好，Ambi！'));
}

main().catch(console.error);
```

## 常用 API

| JS | 作用 |
| --- | --- |
| `LLMEngineConfig.openai(opts)` | OpenAI 兼容后端 |
| `Agent.make(config)` | 初始化可复用 Agent |
| `agent.preamble(text)` | System Prompt |
| `agent.tool(...)` | 注册 JS 工具 |
| `AgentState(id)` | 会话状态 |
| `Pipeline.chatRunner(n)` | 内置执行策略 |
| `Pipeline.custom(...)` | JS 自定义执行策略 |
| `runner.chat(...)`、`runner.chatStream(...)` | 完整或流式结果 |

工具辅助函数、自定义 Engine 和流式示例，可参考当前版本的 [`bindings/node/examples`](https://github.com/Maskviva/Ambi/tree/main/bindings/node/examples)，以及与安装版本对应的 `lib/index.d.ts`。

## 跨语言能力边界

树／枝／叶是 **Rust Core 的设计模型**，不代表 Node 的 `Pipeline.chatRunner` 已经自动支持 `with_leaf()`。采用新原生功能前，应分别确认绑定接口。参见 [设计理念](/zh/advanced/design-philosophy) 与 [Leaf](/zh/guide/leaf)。
