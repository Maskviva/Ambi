# Python 绑定

Python 独立发行包名为 **`ambi-python`**，Python 导入模块名为 **`ambi_python`**。仓库 `bindings/python/pyproject.toml` 声明版本 **0.1.1**，不代表它与 Rust Core 0.4.1 的 API 完全同步。

## 安装

```bash
pip install ambi-python
```

本地开发可安装 `maturin`，然后在 `bindings/python/` 中使用 `maturin develop --release`（需要兼容的 Python/Rust 环境）。

## 最小示例

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
    agent = agent.preamble('你是一个简洁的助手。')
    state = AgentState('py-session-1')
    runner = Pipeline.chat_runner(5)
    print(await runner.chat(agent, state, '你好，Ambi！'))

asyncio.run(main())
```

## 常用接口

| Python | 作用 |
| --- | --- |
| `LLMEngineConfig.openai(...)` | OpenAI 兼容后端配置 |
| `await Agent.make(config)` | 初始化 Agent |
| `agent.preamble(text)` | System Prompt |
| `agent.add_tool(...)` | 注册 Python 工具 |
| `AgentState(session_id)` | 会话状态 |
| `Pipeline.chat_runner(n)` | 内置聊天执行策略 |
| `Pipeline.custom(...)` | Python 自定义 Pipeline |
| `runner.chat(...)`、`runner.chat_stream(...)` | 完整或流式结果 |

工具与自定义 Engine 参考 [`bindings/python/examples`](https://github.com/Maskviva/Ambi/tree/main/bindings/python/examples)，并以实际安装版本的 API 为准。

## Leaf 的支持范围

Rust 0.4.1 的 `ChatRunner::with_leaf()` **不是**本 Python 绑定文档承诺的 API。不能假定 Rust Cargo Feature 自动出现在 Python 绑定中。参见 [设计理念](/zh/advanced/design-philosophy) 和 [Leaf](/zh/guide/leaf)。
