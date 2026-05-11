"""
custom-engine.py  -- Custom Python LLM engine

Demonstrates how to integrate any Python LLM backend.
No API key needed -- the engine is a Python function.
Usage:    python examples/custom-engine.py
"""

import asyncio
import json
from ambi_python import Agent, AgentState, Pipeline, LLMEngineConfig, resolve_request


async def main():
    # Step 1: Define the callback.
    #   - It receives a JSON string with {"request_id": "...", "request": {...}}.
    #   - The callback MUST be SYNCHRONOUS (cannot be async). Start async work
    #     internally and call resolve_request() when done.
    def handler(req_json: str):
        payload = json.loads(req_json)
        request_id = payload["request_id"]
        request = payload["request"]
        print(f"[Custom Engine] Received prompt: {request['formatted_prompt']}")

        # Start async work in the background.
        async def do_work():
            # Replace with e.g. httpx.AsyncClient().post(...)
            fake_response = f"I am a custom Python engine. You said: \"{request['formatted_prompt'][:60]}...\""
            resolve_request(request_id, fake_response)

        asyncio.create_task(do_work())

    # Step 2: Create the engine config with the custom handler.
    engine_config = LLMEngineConfig.custom(
        chat_handler=handler,
        supports_multimodal=False,
    )

    # Step 3: Create the Agent as usual.
    agent = await Agent.make(engine_config)
    agent = agent.preamble("You are a custom Python-powered assistant.")

    # Step 4: Chat with the agent.
    state = AgentState("custom-engine-demo")
    runner = Pipeline.chat_runner(5)

    response = await runner.chat(agent, state, "Hello from Ambi!")
    print("Response:", response)


asyncio.run(main())
