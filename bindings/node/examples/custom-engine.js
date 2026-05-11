// Import the Ambi Agent helpers.
const {Agent, AgentState, Pipeline, LLMEngineConfig} = require('../lib')

// ---- Custom JS LLM engine example ----
// Demonstrates how to integrate any JavaScript LLM backend.
// No API key needed — the engine is a JS function.
// Run:  node examples/custom-engine.js

async function main() {
    // Step 1: Define a custom chat handler.
    //   - It receives (err, argsJson) where argsJson is a JSON string
    //     containing { request_id, request: { formatted_prompt, ... } }.
    //   - The handler must be SYNCHRONOUS (cannot be async). Start async
    //     work internally and call resolveRequest() when done.
    const {resolveRequest} = require('../lib')

    // Step 2: Create the engine config with your custom handler.
    const engineConfig = LLMEngineConfig.custom(
        (err, argsJson) => {
            if (err) throw err
            const {request_id, request} = JSON.parse(argsJson)
            console.log('[Custom Engine] Received prompt:', request.formatted_prompt)

            // Simulate async LLM work — send the result back via resolveRequest.
            ;(async () => {
                // Replace this with a real API call, e.g. fetch('https://…')
                const fakeResponse = `I am a custom JS engine. You said: "${request.formatted_prompt.slice(0, 60)}…"`
                resolveRequest(request_id, fakeResponse)
            })()
        },
        false,  // supportsMultimodal
        null,   // chatStreamHandler (optional)
    )

    // Step 3: Create the Agent as usual.
    const agent = (await Agent.make(engineConfig))
        .preamble('You are a custom JS-powered assistant.')

    // Step 4: Chat with the agent — it will invoke your custom handler.
    const state = new AgentState('custom-engine-demo')
    const runner = Pipeline.chatRunner(5)

    const response = await runner.chat(agent, state, 'Hello from Ambi!')
    console.log('Response:', response)
}

main().catch(console.error)
