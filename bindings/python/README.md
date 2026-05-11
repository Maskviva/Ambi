# Python Bindings for Ambi

A Python native extension for the [Ambi](https://github.com/Maskviva/Ambi) AI Agent framework.

## Installation

### From PyPI (recommended)

```bash
pip install ambi-python
```

### Build from source

Requires [maturin](https://maturin.rs) and a Rust toolchain.

```bash
# Install maturin
pip install maturin

# Build and install locally
cd bindings/python
maturin develop --release
```

Then import as:

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
```

The built `.whl` files will be in `target/wheels/`.

## Examples

Run any example from the `bindings/python` directory:

```bash
cd bindings/python

# Basic cloud-based chat
OPENAI_API_KEY=sk-... python examples/chat-cloud.py

# Streaming output
OPENAI_API_KEY=sk-... python examples/chat-stream.py

# Tool calling
OPENAI_API_KEY=sk-... python examples/custom-tool.py

# Custom chat template
OPENAI_API_KEY=sk-... python examples/custom-template.py

# Custom Python LLM engine
python examples/custom-engine.py

# Memory eviction
OPENAI_API_KEY=sk-... python examples/memory-eviction.py
```

## API Reference

| Python | JS equivalent | Description |
|--------|---------------|-------------|
| `Agent.make(config)` | `Agent.make(config)` | Create an async agent |
| `agent.preamble("...")` | `agent.preamble("...")` | Set system prompt |
| `agent.template("chatml")` | `agent.template(Chatml)` | Set template type |
| `agent.custom_template(...)` | `agent.customTemplate(...)` | Custom template |
| `agent.add_tool(...)` | `agent.tool(...)` | Register a tool |
| `agent.with_standard_formatting()` | `agent.withStandardFormatting()` | Enable standard formatting |
| `agent.with_eviction_strategy(...)` | `agent.withEvictionStrategy(...)` | Set eviction strategy |
| `agent.count_tokens(...)` | `agent.countTokens(...)` | Count tokens |
| `AgentState("id")` | `new AgentState("id")` | Create session state |
| `LLMEngineConfig.openai(...)` | `LLMEngineConfig.openai(...)` | OpenAI engine |
| `LLMEngineConfig.custom(...)` | `LLMEngineConfig.custom(...)` | Custom engine |
| `Pipeline.chat_runner(n)` | `Pipeline.chatRunner(n)` | Chat runner |
| `Pipeline.custom(...)` | `Pipeline.custom(...)` | Custom pipeline |
| `runner.chat(...)` | `runner.chat(...)` | Sync chat |
| `runner.chat_stream(...)` | `runner.chatStream(...)` | Streaming chat |
| `resolve_request(id, result)` | `resolveRequest(id, result)` | Resolve async callback |
| `resolve_pipeline_request(...)` | `resolvePipelineRequest(...)` | Resolve pipeline callback |
