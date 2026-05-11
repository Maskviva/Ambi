// Import the Ambi Agent and configuration helpers.
const {Agent, AgentState, Pipeline, LLMEngineConfig, JsChatTemplateType} = require('../lib')

// ---- Basic cloud-based chat example ----
// 1. Set your API key via the OPENAI_API_KEY environment variable.
// 2. Run:  OPENAI_API_KEY=sk-... node examples/chat-cloud.js

async function main() {
    // Step 1: Read the API key from the environment.
    const apiKey = process.env.OPENAI_API_KEY || 'sk-your-key-here'

    // Step 2: Configure the remote LLM engine (OpenAI-compatible).
    const engineConfig = LLMEngineConfig.openai({
        apiKey,
        baseUrl: 'https://api.openai.com/v1',
        modelName: 'gpt-4o-mini',
        temp: 0.7,
        topP: 0.9,
    })

    // Step 3: Create an Agent with a ChatML template and a system prompt.
    const agent = (await Agent.make(engineConfig))
        .template(JsChatTemplateType.Chatml)
        .preamble('You are a helpful and harmless AI assistant.')

    // Step 4: Create an AgentState and a ChatRunner.
    const state = new AgentState('chat-cloud-demo')
    const runner = Pipeline.chatRunner(5)

    // Step 5: Send a chat message and print the result.
    const response = await runner.chat(agent, state, 'Who are you and what can you do?')
    console.log(response)
}

main().catch(console.error)
