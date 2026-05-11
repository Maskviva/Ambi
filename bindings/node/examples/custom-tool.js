// Import the Ambi Agent and helpers.
const {Agent, AgentState, Pipeline, LLMEngineConfig, tool} = require('../lib')

// ---- Tool calling example ----
// Registers a weather tool that the LLM can invoke autonomously.
// 1. Set your API key via the OPENAI_API_KEY environment variable.
// 2. Run:  OPENAI_API_KEY=sk-... node examples/custom-tool.js

async function main() {
    const apiKey = process.env.OPENAI_API_KEY || 'sk-your-key-here'

    // Step 1: Configure the remote LLM engine.
    const engineConfig = LLMEngineConfig.openai({
        apiKey, baseUrl: 'https://api.openai.com/v1', modelName: 'gpt-4o-mini', temp: 0.7, topP: 0.9,
    })

    // Step 2: Create an Agent and register a tool via the lightweight `tool()` builder.
    //   - `parameters` accepts either full schema objects or shorthand strings.
    //   - The callback receives pre-parsed args and auto-stringifies the return value.
    const agent = (await Agent.make(engineConfig))
        .preamble('You are a weather assistant. Always use the get_weather tool.')
        .tool(tool({
            name: 'get_weather', description: 'Query real-time weather information for a specified city', parameters: {
                city: {
                    type: 'string',
                    description: 'Query real-time weather information for a specified city. City names, for example: Beijing, Shanghai, Shenzhen'
                }, unit: {type: 'string', enum: ['celsius', 'fahrenheit'], description: 'Temperature unit'},
            }, callback: (args) => ({
                city: args.city,
                temperature: Math.round(Math.random() * 35 + 5),
                unit: args.unit ?? 'celsius',
                condition: 'Sunny',
                humidity: Math.round(Math.random() * 60 + 30),
            }),
        }))
        .withStandardFormatting()

    // Step 3: Chat with the agent — it will call the tool automatically.
    const state = new AgentState('tool-demo')
    const runner = Pipeline.chatRunner(5)

    console.log('User: How is the weather in Beijing today?？')
    const response = await runner.chat(agent, state, 'How is the weather in Beijing today?？')
    console.log('Assistant:', response)
}

main().catch(console.error)
