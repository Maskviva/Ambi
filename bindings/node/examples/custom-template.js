// Import the Ambi Agent and helpers.
const {Agent, LLMEngineConfig, deepseekTemplate} = require('../lib')

// ---- Custom chat template example ----
// Shows how to use a built-in template or define a fully custom one.
// Run:  OPENAI_API_KEY=sk-... node examples/custom-template.js

async function main() {
    const apiKey = process.env.OPENAI_API_KEY || 'sk-your-key-here'

    // Step 1: Configure the remote LLM engine.
    const engineConfig = LLMEngineConfig.openai({
        apiKey,
        baseUrl: 'https://api.openai.com/v1',
        modelName: 'gpt-4o-mini',
        temp: 0.7,
        topP: 0.9,
    })

    // --- Option A: Use a built-in template (returned as a JS object) ---
    const builtInTpl = deepseekTemplate()  // also: chatmlTemplate(), llama3Template(), qwenTemplate() …
    console.log('DeepSeek template system prefix:', JSON.stringify(builtInTpl.systemPrefix))

    // --- Option B: Define a fully custom template ---
    const customTpl = {
        systemPrefix: '<|SYS_START|>\n',
        systemSuffix: '\n<|SYS_END|>\n\n',
        userPrefix: '<|HUMAN|>: ',
        userSuffix: '\n',
        assistantPrefix: '<|BOT|>: ',
        assistantSuffix: '\n',
        thinkPrefix: '',
        thinkSuffix: '',
        toolPrefix: '<|TOOL_EXECUTION|>\n',
        toolSuffix: '\n<|END_EXECUTION|>\n',
        toolIdPrefix: '',
        toolIdSuffix: '',
        mediaPlaceholder: '',
    }

    // Step 2: Mount the custom template onto the Agent.
    const _agent = (await Agent.make(engineConfig)).customTemplate(customTpl)

    console.log('Agent created with custom template. Ready for chat.')
    // Use it like: const res = await runner.chat(agent, state, "Hello")
}

main().catch(console.error)
