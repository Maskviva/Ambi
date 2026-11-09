# Python binding

The Python binding is a separate distribution, **`ambi-python`**, with module name **`ambi_python`**. In this repository, `bindings/python/pyproject.toml` declares version **0.1.1**; it does not automatically track the Rust core's 0.4.1 API.

## Install

```bash
pip install ambi-python
```

To develop locally, install `maturin`, then run `maturin develop --release` in `bindings/python/` with a compatible Python/Rust environment.

## Minimal example

```python
import asyncio
import os
from ambi_python import Agent, AgentState, Pipeline, LLMEngineConfig

async def main():
    cfg = LLMEngineConfig.openai(
        api_key=os.environ['OPENAI_API_KEY'],
        base_url='https://api.openai.com/v1',
        model_name='gpt-4o-mini',
        temp=0.7,
        top_p=0.9,
    )
    agent = await Agent.make(cfg)
    agent = agent.preamble('You are a concise assistant.')
    state = AgentState('py-session-1')
    runner = Pipeline.chat_runner(5)
    print(await runner.chat(agent, state, 'Hello, Ambi!'))

asyncio.run(main())
```

## Common Python surface

| Python | Meaning |
| --- | --- |
| `LLMEngineConfig.openai(...)` | Configure OpenAI-compatible API |
| `await Agent.make(config)` | Initialize agent |
| `agent.preamble(text)` | Set system prompt |
| `agent.add_tool(...)` | Register a Python-defined tool |
| `AgentState(session_id)` | Conversation state |
| `Pipeline.chat_runner(n)` | Built-in chat strategy |
| `Pipeline.custom(...)` | Python-provided execution strategy |
| `runner.chat(...)`, `runner.chat_stream(...)` | Full or streaming response |

Follow [`bindings/python/examples`](https://github.com/Maskviva/Ambi/tree/main/bindings/python/examples) and the installed binding's Python API for custom tools and engines.

## Leaf availability

The 0.4.1 Rust `ChatRunner::with_leaf()` method is not a documented method in this Python binding. Do not assume a new Rust Cargo feature becomes available through Python automatically. See [Design philosophy](/advanced/design-philosophy) and [Leaf](/guide/leaf).
