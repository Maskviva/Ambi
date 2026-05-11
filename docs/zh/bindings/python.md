# Python 绑定

Python 绑定让你在 Python 中使用 Ambi，支持 OpenAI 兼容 API、自定义 LLM 引擎、工具调用和流式输出。

## 安装

### 从 PyPI 安装（推荐）

```bash
pip install ambi-python
```

### 从源码构建

需要 [maturin](https://maturin.rs) 和 Rust 工具链。

```bash
git clone https://github.com/Maskviva/Ambi.git
cd Ambi/bindings/python
pip install maturin
maturin develop --release
```

直接导入使用：

```python
from ambi import Agent, AgentState, Pipeline, LLMEngineConfig
```

## 构建与发布

```bash
cd bindings/python

# 构建 wheel
maturin build --release

# 发布到 PyPI
maturin publish --username __token__ --password pypi-xxxxx

# 或使用 twine
maturin build --release
pip install twine
twine upload target/wheels/ambi_python-*.whl
```

## 快速开始

```python
import asyncio
from ambi import Agent, AgentState, Pipeline, LLMEngineConfig

async def main():
    # 1. 配置引擎
    config = LLMEngineConfig.openai(
        api_key="sk-...",
        base_url="https://api.openai.com/v1",
        model_name="gpt-4o-mini",
        temp=0.7,
        top_p=0.9,
    )

    # 2. 创建 Agent
    agent = await Agent.make(config)
    agent = agent.template("chatml").preamble("You are a helpful assistant.")

    # 3. 开始对话
    state = AgentState("session-1")
    runner = Pipeline.chat_runner(5)
    reply = await runner.chat(agent, state, "Hello!")

asyncio.run(main())
```

## API 参考

| Python API | JS 对应 | 说明 |
|-----------|---------|------|
| `await Agent.make(config)` | `await Agent.make(config)` | 创建 Agent |
| `agent.preamble(text)` | `agent.preamble(text)` | 设置系统提示词 |
| `agent.template(type_str)` | `agent.template(type)` | 模板类型（"chatml", "llama3", …） |
| `agent.custom_template(...)` | `agent.customTemplate(...)` | 自定义模板（13 个参数） |
| `agent.add_tool(name, desc, params_json, cb)` | `agent.tool(tool(...))` | 注册工具 |
| `agent.with_standard_formatting()` | `agent.withStandardFormatting()` | 启用标准格式化 |
| `agent.with_eviction_strategy(...)` | `agent.withEvictionStrategy(...)` | 上下文驱逐策略 |
| `agent.max_iterations(n)` | `agent.maxIterations(n)` | 最大工具迭代次数 |
| `agent.with_tool_tags(s, e)` | `agent.withToolTags(s, e)` | 自定义工具标签 |
| `agent.count_tokens(text)` | `agent.countTokens(text)` | Token 计数 |
| `AgentState(id)` | `new AgentState(id)` | 会话状态 |
| `LLMEngineConfig.openai(...)` | `LLMEngineConfig.openai(...)` | OpenAI 引擎 |
| `LLMEngineConfig.custom(handler)` | `LLMEngineConfig.custom(handler)` | 自定义 Python 引擎 |
| `Pipeline.chat_runner(n)` | `Pipeline.chatRunner(n)` | 对话运行器 |
| `Pipeline.custom(handler)` | `Pipeline.custom(handler)` | 自定义 Python 管道 |
| `await runner.chat(...)` | `await runner.chat(...)` | 同步对话 |
| `await runner.chat_stream(...)` | `await runner.chatStream(...)` | 流式对话 |
| `await stream.next_chunk()` | `await stream.nextChunk()` | 读取下一个 token |
| `resolve_request(id, result)` | `resolveRequest(id, result)` | 解析异步回调 |

## 工具注册

手动构建 JSON Schema 或使用 Python 辅助函数，然后调用 `add_tool()`：

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
    "description": "查询指定城市的实时天气",
    "parameters": {"city": {"type": "string", "description": "城市名称"}},
    "callback": lambda args: {"temperature": 25, "condition": "晴"},
})

agent = agent.add_tool(*tool_args)
```

## 自定义 LLM 引擎

从任意 Python 可调用对象创建自定义引擎。处理函数必须是**同步的**——在内部启动异步工作，完成后调用 `resolve_request()`：

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

## 流式响应

```python
stream = await runner.chat_stream(agent, state, "讲个故事")
while True:
    chunk = await stream.next_chunk()
    if chunk is None:
        break
    print(chunk, end="", flush=True)
```

## 模板字符串

内置模板以返回字典的函数形式提供：

```python
from ambi import chatml_template, deepseek_template, llama3_template

tpl = deepseek_template()
print(tpl["system_prefix"])  # <|SYS_START|>\n
```

可用：`chatml_template`, `llama3_template`, `gemma_template`, `phi3_template`, `zephyr_template`, `deepseek_template`, `qwen_template`, `mistral_template`, `llama2_template`。
