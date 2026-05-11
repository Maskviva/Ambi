# Python Binding

The Python binding lets you use Ambi from Python with full access to OpenAI-compatible APIs, custom LLM engines, tool calling, and streaming.

## Installation

### From PyPI (recommended)

```bash
pip install ambi-python
```

### Build from source

Requires [maturin](https://maturin.rs) and a Rust toolchain.

```bash
git clone https://github.com/Maskviva/Ambi.git
cd Ambi/bindings/python

# Install maturin if needed
pip install maturin

# Build and install the native module
maturin develop --release
```

Import directly:

```python
from ambi import Agent, AgentState, Pipeline, LLMEngineConfig
```

## Build & Publish

```bash
cd bindings/python

# Build wheel
maturin build --release

# Publish to PyPI
maturin publish --username __token__ --password pypi-xxxxx

# Or use twine
maturin build --release
pip install twine
twine upload target/wheels/ambi_python-*.whl
```

## Quick Start

```python
import asyncio
from ambi import Agent, AgentState, Pipeline, LLMEngineConfig

async def main():
    # 1. Configure the engine
    config = LLMEngineConfig.openai(
        api_key="sk-...",
        base_url="https://api.openai.com/v1",
        model_name="gpt-4o-mini",
        temp=0.7,
        top_p=0.9,
    )

    # 2. Create the agent
    agent = await Agent.make(config)
    agent = agent.template("chatml").preamble("You are a helpful assistant.")

    # 3. Chat
    state = AgentState("session-1")
    runner = Pipeline.chat_runner(5)
    reply = await runner.chat(agent, state, "Hello!")

asyncio.run(main())
```

## API Reference

| Python API | JS Equivalent | Description |
|-----------|---------------|-------------|
| `await Agent.make(config)` | `await Agent.make(config)` | Create an agent |
| `agent.preamble(text)` | `agent.preamble(text)` | Set system prompt |
| `agent.template(type_str)` | `agent.template(type)` | Template type ("chatml", "llama3", …) |
| `agent.custom_template(...)` | `agent.customTemplate(...)` | Custom template (13 kwargs) |
| `agent.add_tool(name, desc, params_json, cb)` | `agent.tool(tool(...))` | Register a tool |
| `agent.with_standard_formatting()` | `agent.withStandardFormatting()` | Enable standard formatting |
| `agent.with_eviction_strategy(...)` | `agent.withEvictionStrategy(...)` | Memory eviction |
| `agent.max_iterations(n)` | `agent.maxIterations(n)` | Max tool iterations |
| `agent.with_tool_tags(s, e)` | `agent.withToolTags(s, e)` | Custom tool tags |
| `agent.count_tokens(text)` | `agent.countTokens(text)` | Token counting |
| `AgentState(id)` | `new AgentState(id)` | Session state |
| `LLMEngineConfig.openai(...)` | `LLMEngineConfig.openai(...)` | OpenAI engine |
| `LLMEngineConfig.custom(handler)` | `LLMEngineConfig.custom(handler)` | Custom Python engine |
| `Pipeline.chat_runner(n)` | `Pipeline.chatRunner(n)` | Chat runner |
| `Pipeline.custom(handler)` | `Pipeline.custom(handler)` | Custom Python pipeline |
| `await runner.chat(...)` | `await runner.chat(...)` | Sync chat |
| `await runner.chat_stream(...)` | `await runner.chatStream(...)` | Streaming chat |
| `await stream.next_chunk()` | `await stream.nextChunk()` | Read next token |
| `resolve_request(id, result)` | `resolveRequest(id, result)` | Resolve async callback |

## Tool Registration

Build the JSON schema manually or with a small Python helper, then call `add_tool()`:

```python
import json

def build_tool(options):
    name = options["name"]
    description = options["description"]
    required = list(options["parameters"].keys())
    properties = {}
    for key, val in options["parameters"].items():
        if isinstance(val, list):
            properties[key] = {"type": "string", "enum": val, "description": key}
        elif isinstance(val, str):
            properties[key] = {"type": val, "description": key}
        else:
            properties[key] = val
    params_json = json.dumps({"type": "object", "properties": properties, "required": required})

    def wrapped(args_json):
        args = json.loads(args_json)
        result = options["callback"](args)
        return result if isinstance(result, str) else json.dumps(result)

    return name, description, params_json, wrapped

tool_args = build_tool({
    "name": "get_weather",
    "description": "Query real-time weather for a city",
    "parameters": {"city": {"type": "string", "description": "City name"}},
    "callback": lambda args: {"temperature": 25, "condition": "Sunny"},
})

agent = agent.add_tool(*tool_args)
```

## Custom LLM Engine

Create a custom engine from any Python callable. The handler must be **synchronous** — start async work inside and call `resolve_request()` when done:

```python
import asyncio, json
from ambi import resolve_request

def handler(req_json: str):
    payload = json.loads(req_json)
    request_id = payload["request_id"]
    request = payload["request"]

    async def do_work():
        result = await my_async_llm_call(request["formatted_prompt"])
        resolve_request(request_id, result)

    asyncio.create_task(do_work())

config = LLMEngineConfig.custom(chat_handler=handler, supports_multimodal=False)
```

## Streaming

```python
stream = await runner.chat_stream(agent, state, "Tell me a story")
while True:
    chunk = await stream.next_chunk()
    if chunk is None:
        break
    print(chunk, end="", flush=True)
```

## Template Strings

Built-in templates are available as functions returning dicts:

```python
from ambi import chatml_template, deepseek_template, llama3_template

tpl = deepseek_template()
print(tpl["system_prefix"])  # <|SYS_START|>\n
```

Available: `chatml_template`, `llama3_template`, `gemma_template`, `phi3_template`, `zephyr_template`, `deepseek_template`, `qwen_template`, `mistral_template`, `llama2_template`.
