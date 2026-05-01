// @ts-check
//
// Bypass the Agent pipeline and call the engine directly.
// Useful when you want full control over the request payload —
// write your own ReAct loop, inject custom history, etc.
//
// Run:
//   OPENAI_API_KEY=sk-... node examples/direct-engine.js

const { JsEngine, JsChatTemplateType } = require('..');

function main() {
  const engine = JsEngine.createOpenai({
    apiKey: process.env.OPENAI_API_KEY ?? 'sk-your-key',
    baseUrl: 'https://api.openai.com/v1',
    modelName: 'gpt-4o-mini',
    temp: 0.7,
    topP: 0.9,
  });

  // Build a raw LLM request — exactly what the engine natively expects.
  const request = {
    systemPrompt: 'You are a helpful assistant.',
    history: [
      { role: 'user', content: 'What is the capital of France?' },
    ],
    tools: [],
    toolPrompt: '',
    formattedPrompt: '', // the framework fills this for local engines
    toolTags: ['<tool_call>', '</tool_call>'],
    images: [],
  };

  engine.chat(request).then((reply) => {
    console.log(reply);
  });
}

main();
