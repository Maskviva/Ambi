// @ts-check
//
// Streaming agent — each token fires a callback as it arrives.
// Mirror of examples/chat_stream.rs.
//
// Run:
//   OPENAI_API_KEY=sk-... node examples/chat-stream.js

const { JsEngine, JsAgent, JsAgentState, JsChatRunner } = require('..');

function main() {
  const engine = JsEngine.createOpenai({
    apiKey: process.env.OPENAI_API_KEY ?? 'sk-your-key',
    baseUrl: 'https://api.openai.com/v1',
    modelName: 'gpt-4o-mini',
    temp: 0.7,
    topP: 0.9,
  });

  // Agent + state (must be ready before the sync chatStream call)
  JsAgent.make(engine).then((agent) => {
    const state = new JsAgentState();
    const runner = new JsChatRunner();

    runner.chatStream(
      agent,
      state,
      'Tell me a short story about a Rustacean exploring Node.js.',
      // onToken — called for every text chunk
      (token) => process.stdout.write(token),
      // onComplete — stream finished
      () => process.stdout.write('\n'),
      // onError — something went wrong
      (err) => {
        console.error('\nStream error:', err);
        process.exit(1);
      },
    );
  });
}

main();
