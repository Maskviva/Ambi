// @ts-check
//
// Multi-turn conversation — the agent remembers previous messages
// through `AgentState`. Mirror of the multi-turn section in the docs.
//
// Run:
//   OPENAI_API_KEY=sk-... node examples/multi-turn.js

const { JsEngine, JsAgent, JsAgentState, JsChatRunner } = require('..');

async function main() {
  const engine = JsEngine.createOpenai({
    apiKey: process.env.OPENAI_API_KEY ?? 'sk-your-key',
    baseUrl: 'https://api.openai.com/v1',
    modelName: 'gpt-4o-mini',
    temp: 0.7,
    topP: 0.9,
  });

  const agent = await JsAgent.make(engine)
    .preamble('You are a helpful assistant.')
    .withStandardFormatting();

  const state = new JsAgentState();
  const runner = new JsChatRunner();

  // Turn 1 — introduce yourself
  const reply1 = await runner.chat(agent, state, 'My name is Alice.');
  console.log('Turn 1:', reply1);

  // Turn 2 — the agent should remember "Alice"
  const reply2 = await runner.chat(agent, state, "What's my name?");
  console.log('Turn 2:', reply2);

  // Turn 3 — ask something else
  const reply3 = await runner.chat(agent, state, 'What is 2 + 2?');
  console.log('Turn 3:', reply3);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
