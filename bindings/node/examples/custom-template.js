// @ts-check
//
// Custom chat template — every prefix and suffix, your way.
// Mirror of examples/custom_chat_template.rs.
//
// Some fine-tuned models expect very specific delimiters. Instead of
// fighting the framework, you just build a `JsChatTemplate` object
// and hand it to the agent.
//
// Run:
//   OPENAI_API_KEY=sk-... node examples/custom-template.js

const {
  JsEngine,
  JsAgent,
  JsAgentState,
  JsChatRunner,
} = require('..');

async function main() {
  const engine = JsEngine.createOpenai({
    apiKey: process.env.OPENAI_API_KEY ?? 'sk-your-key',
    baseUrl: 'https://api.openai.com/v1',
    modelName: 'gpt-4o-mini',
    temp: 0.7,
    topP: 0.9,
  });

  // Build a template for a hypothetical model that uses
  // <|SYS|> / <|USER|> / <|BOT|> markers.
  const template = {
    systemPrefix: '<|SYS|>\n',
    systemSuffix: '\n<|/SYS|>\n\n',
    userPrefix: '<|USER|> ',
    userSuffix: '\n',
    assistantPrefix: '<|BOT|> ',
    assistantSuffix: '<|END|>\n',
    thinkPrefix: '',
    thinkSuffix: '',
    toolPrefix: '<|TOOL|>\n',
    toolSuffix: '\n<|/TOOL|>\n',
    toolIdPrefix: '',
    toolIdSuffix: '',
    mediaPlaceholder: '',
  };

  const agent = await JsAgent.make(engine)
    .setCustomTemplate(template)
    .preamble('You are a custom-templated assistant.');

  const state = new JsAgentState();
  const runner = new JsChatRunner();

  const reply = await runner.chat(agent, state, 'Hello!');
  console.log(reply);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
