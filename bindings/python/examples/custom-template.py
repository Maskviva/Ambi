"""
custom-template.py  -- Custom chat template

Shows how to use a built-in template or define a fully custom one.
Usage:    python examples/custom-template.py
"""

import asyncio
import os
from ambi_python import Agent, LLMEngineConfig, deepseek_template


async def main():
    api_key = os.environ.get("OPENAI_API_KEY", "sk-your-key-here")

    engine_config = LLMEngineConfig.openai(
        api_key=api_key,
        base_url="https://api.openai.com/v1",
        model_name="gpt-4o-mini",
        temp=0.7,
        top_p=0.9,
    )

    # --- Option A: Use a built-in template ---
    built_in = deepseek_template()
    print("DeepSeek template system_prefix:", built_in["system_prefix"])

    # --- Option B: Define a fully custom template ---
    # All 13 template fields must be provided as keyword arguments.
    agent = await Agent.make(engine_config)
    agent = agent.custom_template(
        system_prefix="<|SYS_START|>\n",
        system_suffix="\n<|SYS_END|>\n\n",
        user_prefix="<|HUMAN|>: ",
        user_suffix="\n",
        assistant_prefix="<|BOT|>: ",
        assistant_suffix="\n",
        think_prefix="",
        think_suffix="",
        tool_prefix="<|TOOL_EXECUTION|>\n",
        tool_suffix="\n<|END_EXECUTION|>\n",
        tool_id_prefix="",
        tool_id_suffix="",
        media_placeholder="",
    )

    print("Agent created with custom template. Ready for chat.")


asyncio.run(main())
