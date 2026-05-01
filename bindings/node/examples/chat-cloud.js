// @ts-check
//
// Minimal cloud agent — the Node equivalent of examples/chat_cloud.rs.
//
// Run:
//   OPENAI_API_KEY=sk-... node examples/chat-cloud.js

const { JsEngine, JsAgent, JsAgentState, JsChatRunner } = require('..');

async function main() {
  // 1. Build the engine
  const engine = JsEngine.createOpenai({
    apiKey: process.env.OPENAI_API_KEY ?? 'sk-your-key',
    baseUrl: 'https://api.openai.com/v1',
    modelName: 'gpt-4o-mini',
    temp: 0.7,
    topP: 0.9,
  });

  // 2. Create the agent (builder pattern, same as Rust)
  const agent = await JsAgent.make(engine)
    .preamble('You are a helpful and harmless AI assistant.')
    .withStandardFormatting();

  // 3. Per-conversation state
  const state = new JsAgentState();

  // 4. ReAct loop
  const runner = new JsChatRunner();
  const reply = await runner.chat(
    agent,
    state,
    'Who are you and what can you do?',
  );

  console.log(reply);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
