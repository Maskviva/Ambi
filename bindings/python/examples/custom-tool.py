"""
custom-tool.py  -- Tool calling

Registers a weather tool that the LLM can invoke autonomously.
Requires: OPENAI_API_KEY env var or edit the placeholder below.
Usage:    python examples/custom-tool.py
"""

import asyncio
import json
import os
from ambi_python import Agent, AgentState, Pipeline, LLMEngineConfig


def build_tool(options: dict):
    """Lightweight tool builder (mirrors JS `tool()` helper)."""
    name = options["name"]
    description = options["description"]
    raw = options["parameters"]
    required = list(raw.keys())
    properties = {}
    for key, val in raw.items():
        if isinstance(val, list):
            properties[key] = {"type": "string", "enum": val, "description": key}
        elif isinstance(val, str):
            properties[key] = {"type": val, "description": key}
        else:
            properties[key] = val
            if val.get("required") is not False and key not in required:
                required.append(key)
    params_json = json.dumps({
        "type": "object",
        "properties": properties,
        "required": required or None,
    })
    callback = options["callback"]

    def wrapped(args_json: str) -> str:
        args = json.loads(args_json)
        result = callback(args)
        return result if isinstance(result, str) else json.dumps(result)

    return name, description, params_json, wrapped, \
        options.get("timeout_secs"), options.get("max_retries"), options.get("is_idempotent", True)


async def main():
    api_key = os.environ.get("OPENAI_API_KEY", "sk-your-key-here")

    engine_config = LLMEngineConfig.openai(
        api_key=api_key,
        base_url="https://api.openai.com/v1",
        model_name="gpt-4o-mini",
        temp=0.7,
        top_p=0.9,
    )

    # Build tool arguments via the helper, then pass to agent.add_tool()
    tool_args = build_tool({
        "name": "get_weather",
        "description": "Query real-time weather information for a specified city",
        "parameters": {
            "city": {
                "type": "string",
                "description": "City name, e.g. Beijing, Shanghai, Shenzhen",
            },
            "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"],
                "description": "Temperature unit",
            },
        },
        "callback": lambda args: {
            "city": args["city"],
            "temperature": 25,
            "unit": args.get("unit", "celsius"),
            "condition": "Sunny",
            "humidity": 50,
        },
    })

    agent = await Agent.make(engine_config)
    agent = agent.preamble("You are a weather assistant. Always use the get_weather tool.") \
                 .add_tool(*tool_args) \
                 .with_standard_formatting()

    state = AgentState("tool-demo")
    runner = Pipeline.chat_runner(5)

    print("User: How is the weather in Beijing today?")
    response = await runner.chat(agent, state, "How is the weather in Beijing today?")
    print("Assistant:", response)


asyncio.run(main())
