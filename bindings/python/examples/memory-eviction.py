"""
memory-eviction.py  -- Memory eviction handler

The Agent will evict old messages when the context exceeds maxSafeTokens.
Usage:    python examples/memory-eviction.py
"""

import asyncio
import json
import os
from ambi_python import Agent, LLMEngineConfig


async def main():
    api_key = os.environ.get("OPENAI_API_KEY", "sk-your-key-here")

    engine_config = LLMEngineConfig.openai(
        api_key=api_key,
        base_url="https://api.openai.com/v1",
        model_name="gpt-4o-mini",
        temp=0.7,
        top_p=0.9,
    )

    # Note: on_evict is not yet exposed in the Python binding.
    # The eviction strategy is still configurable.
    agent = await Agent.make(engine_config)
    agent = agent.template("chatml") \
                 .preamble("You are a helpful AI assistant.") \
                 .with_eviction_strategy(max_safe_tokens=50)

    print("Agent initialized with memory eviction strategy.")
    print("Start a long conversation -- old messages will be evicted automatically.")


asyncio.run(main())
