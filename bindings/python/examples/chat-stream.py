"""
chat-stream.py  -- Streaming output (token by token)

Requires: OPENAI_API_KEY env var or edit the placeholder below.
Usage:    python examples/chat-stream.py
"""

import asyncio
import os
from ambi_python import Agent, AgentState, Pipeline, LLMEngineConfig


async def main():
    api_key = os.environ.get("OPENAI_API_KEY", "sk-your-key-here")

    # Step 1: Configure the remote LLM engine.
    engine_config = LLMEngineConfig.openai(
        api_key=api_key,
        base_url="https://api.openai.com/v1",
        model_name="gpt-4o-mini",
        temp=0.7,
        top_p=0.9,
    )

    # Step 2: Create a minimal Agent.
    agent = await Agent.make(engine_config)

    # Step 3: Create an AgentState and a ChatRunner.
    state = AgentState("stream-demo")
    runner = Pipeline.chat_runner(5)

    # Step 4: Initiate a streaming chat request.
    stream = await runner.chat_stream(agent, state, "Who are you and what can you do?")

    # Step 5: Iterate over the chunks as they arrive.
    while True:
        chunk = await stream.next_chunk()
        if chunk is None:
            break
        print(chunk, end="", flush=True)
    print()


asyncio.run(main())
