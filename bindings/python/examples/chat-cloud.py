"""
chat-cloud.py  -- Basic cloud-based chat (OpenAI-compatible API)

Requires: OPENAI_API_KEY env var or edit the placeholder below.
Usage:    python examples/chat-cloud.py
"""

import asyncio
import os
from ambi_python import Agent, AgentState, Pipeline, LLMEngineConfig


async def main():
    # Step 1: Read the API key from the environment.
    api_key = os.environ.get("OPENAI_API_KEY", "sk-your-key-here")

    # Step 2: Configure the remote LLM engine (OpenAI-compatible).
    engine_config = LLMEngineConfig.openai(
        api_key=api_key,
        base_url="https://api.openai.com/v1",
        model_name="gpt-4o-mini",
        temp=0.7,
        top_p=0.9,
    )

    # Step 3: Create an Agent with a ChatML template and a system prompt.
    agent = await Agent.make(engine_config)
    agent = agent.template("chatml").preamble("You are a helpful and harmless AI assistant.")

    # Step 4: Create an AgentState and a ChatRunner.
    state = AgentState("chat-cloud-demo")
    runner = Pipeline.chat_runner(5)

    # Step 5: Send a chat message and print the result.
    response = await runner.chat(agent, state, "Who are you and what can you do?")
    print(response)


asyncio.run(main())
