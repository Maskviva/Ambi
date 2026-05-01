# Node.js Examples

Run any example with `node examples/<file>.js` from the `bindings/node` directory.

```bash
cd bindings/node

# Basic chat
OPENAI_API_KEY=sk-... node examples/chat-cloud.js

# Streaming
OPENAI_API_KEY=sk-... node examples/chat-stream.js

# Custom template
node examples/custom-template.js
```

Each example stands on its own — they all import from the local `index.js`.
