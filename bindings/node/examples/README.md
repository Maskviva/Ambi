# Node.js Examples

Run any example with `node examples/<file>` from the `bindings/node` directory.

```bash
cd bindings/node

# Basic cloud-based chat (OpenAI-compatible API)
OPENAI_API_KEY=sk-... node examples/chat-cloud.js

# Streaming output (token-by-token)
OPENAI_API_KEY=sk-... node examples/chat-stream.js

# Tool calling — LLM autonomously invokes your JS function
OPENAI_API_KEY=sk-... node examples/custom-tool.js

# Custom chat template
OPENAI_API_KEY=sk-... node examples/custom-template.js

# Bring your own LLM engine (any JS async function)
node examples/custom-engine.js

# Memory eviction handler (fires when context window is full)
OPENAI_API_KEY=sk-... node examples/memory-eviction.js

# Local GGUF model via node-llama-cpp (see lib/llama.js)
node examples/local-rag.mjs
```

Each example stands on its own — they all import from `../lib`.

| File | Rust counterpart | Feature showcased |
|------|-----------------|-------------------|
| `chat-cloud.js` | `examples/chat_cloud.rs` | LLMEngineConfig.openai, Agent.make, Pipeline.chatRunner.chat |
| `chat-stream.js` | `examples/chat_stream.rs` | Pipeline.chatRunner.chatStream, stream iteration |
| `custom-tool.js` | `examples/custom_tool.rs` | tool() helper, agent.tool, agent.withStandardFormatting |
| `custom-template.js` | `examples/custom_chat_template.rs` | Built-in & custom chat templates |
| `custom-engine.js` | `examples/custom_model_back.rs` | LLMEngineConfig.custom, resolveRequest |
| `memory-eviction.js` | `examples/memory_eviction_handler.rs` | withEvictionStrategy, onEvict |
| `local-rag.mjs` | — | Local GGUF model via node-llama-cpp |
