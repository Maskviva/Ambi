// Import the Ambi Agent and configuration helpers.
const {Agent, AgentState, Pipeline, LLMEngineConfig} = require('../lib')

// ---- Streaming chat example ----
// 1. Set your API key via the OPENAI_API_KEY environment variable.
// 2. Run:  OPENAI_API_KEY=sk-... node examples/chat-stream.js

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

    // Step 3: Create a minimal Agent (no template/preamble needed for basic chat).
    const agent = await Agent.make(engineConfig)

    // Step 4: Create an AgentState and a ChatRunner.
    const state = new AgentState('stream-demo')
    const runner = Pipeline.chatRunner(5)

    // Step 5: Initiate a streaming chat request.
    // `chatStream()` returns a ChatStream object that yields tokens one-by-one.
    const stream = await runner.chatStream(agent, state, 'Who are you and what can you do?')

    // Step 6: Iterate over the chunks as they arrive.
    for (let chunk = await stream.nextChunk(); chunk !== null; chunk = await stream.nextChunk()) {
        process.stdout.write(chunk)
    }
    console.log()
}

main().catch(console.error)
