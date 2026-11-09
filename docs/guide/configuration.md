# Configuration and feature boundaries

Ambi's configuration reflects its design: choose the *mechanism* (engine, template, tools), the *execution policy* (`Pipeline`), and the *state* independently.

## Cargo features (0.4.1)

| Feature | Purpose | Default |
| --- | --- | --- |
| `openai-api` | OpenAI-compatible API backend | **On** |
| `llama-cpp` | Local GGUF inference (native only) | Off |
| `mtmd` | llama.cpp multimodal support | Off |
| `cuda`, `vulkan`, `metal`, `rocm` | Hardware acceleration; choose one appropriate backend | Off |
| `dynamic-link` | Dynamic llama.cpp linking | Off |
| `macro` | Re-export `ambi-macros` | Off |
| `sub-agent` | Compile Leaf support for `ChatRunner` | Off |

Enabling `sub-agent` does **not** enable Leaf at runtime; use `ChatRunner::with_leaf()`.

## Agent configuration

`Agent::make(cfg)` initializes shared inference resources. Fluent methods configure the agent:

```rust
use ambi::config::EvictionStrategy;

let agent = Agent::make(engine_config).await?
    .preamble("You are a helpful assistant.")
    .max_iterations(8)
    .with_eviction_strategy(EvictionStrategy { max_safe_tokens: 4096 })
    .with_standard_formatting();
```

Defaults from `AgentConfig`: empty system prompt, ChatML template, 10 iterations, and 8,000 approximate safe context tokens. `ChatRunner::default()` limits simultaneous tool calls **inside its loop** to five. These are not global process budgets.

## OpenAI-compatible model config

```rust
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;

let cfg = OpenAIEngineConfig::create(api_key, "your-model")
    .base_url("https://your-endpoint.example/v1".into())
    .temp(0.1)
    .top_p(1.0)
    .with_vision(false);
```

`create()` defaults to the OpenAI URL, temperature `0.0`, top-p `1.0`, and sets `supports_vision` based on recognized model-name prefixes. For an unrecognized endpoint that really supports image input, set `.with_vision(true)` explicitly. This flag is a declaration of capability, not model capability probing.

The OpenAI Engine has an internal request concurrency ceiling (default 32) and per-request timeout (default 300 seconds); those are distinct from `ChatRunner` tool concurrency.

## llama.cpp model config

```rust
use ambi::llm::providers::llama_cpp::config::LlamaEngineConfig;
let cfg = LlamaEngineConfig::builder("./models/model.gguf")
    .n_ctx(4096)
    .n_tokens(512)
    .max_tokens(1024)
    .build();
```

The builder starts from conservative CPU defaults. The model file must exist. Validation rejects `n_ctx = 0`, batch sizes outside `1..=n_ctx`, and nonpositive `max_tokens`. GPU settings require a compatible build. More: [Native platforms](/platform/native).

## Leaf configuration

With `sub-agent` enabled, `LeafConfig` can override the leaf's tool name, description, prompt, iteration limit, inner-loop tool concurrency and timeout. It does not provide a separate engine registry or a built-in tool allow-list. See [Leaf](/guide/leaf).

## Error and budget semantics

An eviction budget is **not** a guaranteed exact context window; model templates, dynamic context, retrieved documents, image tokens and provider-specific accounting vary. Timeouts and non-idempotent retry prevention do not roll back external side effects. Make those guarantees at the application layer where required.
