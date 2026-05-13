# Configuration

## AgentConfig

The `AgentConfig` struct is created internally when you call `Agent::make()`. Defaults are sensible for most use cases:

```rust
pub struct AgentConfig {
    pub system_prompt: String,              // default: ""
    pub template: ChatTemplate,             // default: Chatml
    pub max_iterations: usize,              // default: 10
    pub eviction_strategy: EvictionStrategy, // default: 8K tokens
}
```

You control these via builder methods, not by constructing `AgentConfig` directly.

## EvictionStrategy

Controls when and how old messages are removed from context:

```rust
pub struct EvictionStrategy {
    pub max_safe_tokens: usize,  // default: 8000
}
```

When `total_tokens + prompt_overhead > max_safe_tokens`, the framework pops the oldest messages (FIFO) until the budget fits.

```rust
let agent = Agent::make(config).await?
    .with_eviction_strategy(EvictionStrategy { max_safe_tokens: 4096 });
```

The default of 8K is a rough safe point for 8K-context models. For 128K models you might set it to 64K or higher. The exact value depends on how much output room you need.

## LLMEngineConfig

This is the enum you pass to `Agent::make()`:

```rust
pub enum LLMEngineConfig {
    #[cfg(feature = "openai-api")]
    OpenAI(OpenAIEngineConfig),
    #[cfg(feature = "llama-cpp")]
    Llama(LlamaEngineConfig),
    Custom(Box<dyn LLMEngineTrait>),
}
```

### OpenAI config

```rust
OpenAIEngineConfig {
    api_key: String,
    base_url: String,    // "https://api.openai.com/v1"
    model_name: String,  // "gpt-4o"
    temp: f32,           // 0.0 - 2.0
    top_p: f32,          // 0.0 - 1.0
}
```

`base_url` can point to any OpenAI-compatible endpoint (DeepSeek, Ollama with OpenAI adapter, etc.).

### Llama.cpp config

```rust
LlamaEngineConfig {
    model_path: String,              // path to .gguf file
    mmproj_path: Option<String>,     // external vision projector (e.g., mmproj-model-f16.gguf)
    integrated_vision: bool,         // whether the model has native vision capabilities
    max_tokens: i32,                 // max tokens to predict
    buffer_size: usize,              // batch buffer size for piece decoding
    use_gpu: bool,                   // offload layers to GPU
    n_gpu_layers: u32,               // how many layers to offload to GPU
    n_ctx: u32,                      // context window size
    n_tokens: usize,                 // batch size for prompt processing
    n_seq_max: i32,                  // max sequences in a batch
    penalty_last_n: i32,             // past tokens to consider for penalties
    penalty_repeat: f32,             // repetition penalty
    penalty_freq: f32,               // frequency penalty
    penalty_present: f32,            // presence penalty
    temp: f32,                       // temperature (0.0 – 2.0)
    top_p: f32,                      // nucleus sampling threshold
    seed: u32,                       // RNG seed for deterministic generation
    min_keep: usize,                 // min-keep sampling boundary
}
```

Validation runs at load time – if required fields are missing or out of range, you get an `EngineError` immediately rather than a cryptic crash mid-inference.

## Feature flags

```toml
[dependencies]
ambi = { version = "0.3", default-features = false, features = ["openai-api"] }
```

| Feature     | What it enables                                   | Dependencies                   |
|-------------|---------------------------------------------------|--------------------------------|
| `openai-api`| OpenAI-compatible cloud backend                   | `async-openai`                 |
| `llama-cpp` | Local inference via llama.cpp                     | `llama-cpp-2`, `llama-cpp-sys-2` |
| `cuda`      | CUDA acceleration (implies llama-cpp)             | + CUDA SDK                     |
| `vulkan`    | Vulkan acceleration                               | + Vulkan SDK                   |
| `metal`     | Apple Metal acceleration                          | + Metal framework               |
| `rocm`      | AMD ROCm acceleration                             | + ROCm                         |
| `mtmd`      | Multimodal support for Llama (VLM)                | + `base64`                     |

You cannot enable more than one GPU backend at once – there's a compile-time `compile_error!` guard for this.

## Adding to the runtime requirement

```toml
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time", "macros"] }
```

See [native platform](/platform/native) for details.
