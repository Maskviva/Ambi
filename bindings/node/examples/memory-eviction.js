// Import the Ambi Agent and helpers.
const {Agent, LLMEngineConfig, JsChatTemplateType} = require('../lib')

// ---- Memory eviction handler example ----
// The Agent will evict old messages when the context exceeds maxSafeTokens.
// Run:  OPENAI_API_KEY=sk-... node examples/memory-eviction.js

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

    // Step 2: Create an Agent with a strict eviction strategy.
    const agent = (await Agent.make(engineConfig))
        .template(JsChatTemplateType.Chatml)
        .preamble('You are a helpful AI assistant.')
        // Keep the context window small (50 tokens) to trigger eviction early.
        .withEvictionStrategy({maxSafeTokens: 50})
        // Register a callback that fires when messages are evicted.
        .onEvict((_err, messagesJson) => {
            const msgs = JSON.parse(messagesJson)
            console.log(`\n[Memory Manager] Evicting ${msgs.length} old message(s)…`)
            for (let i = 0; i < msgs.length; i++) {
                console.log(`  -> Message #${i}: role=${msgs[i].role}, ${msgs[i].content.length} chars`)
            }
            console.log('[Memory Manager] Archiving complete.\n')
        })

    console.log('Agent initialized with memory eviction handler.')
    console.log('Start a long conversation — the handler will fire when the context is full.')
}

main().catch(console.error)
