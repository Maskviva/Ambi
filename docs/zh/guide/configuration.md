# 配置详解

## AgentConfig

`AgentConfig` 在调用 `Agent::make()` 时内部创建。默认值对大多数场景够用：

```rust
pub struct AgentConfig {
    pub system_prompt: String,              // 默认：""
    pub template: ChatTemplate,             // 默认：Chatml
    pub max_iterations: usize,              // 默认：10
    pub eviction_strategy: EvictionStrategy, // 默认：8K tokens
}
```

通过 builder 方法控制，不直接构造 `AgentConfig`。

## EvictionStrategy

控制什么时候把旧消息从上下文中移除：

```rust
pub struct EvictionStrategy {
    pub max_safe_tokens: usize,  // 默认：8000
}
```

当 `total_tokens + prompt_overhead > max_safe_tokens` 时，从最早的消息开始弹出（FIFO），直到预算够用。

```rust
let agent = Agent::make(config).await?
    .with_eviction_strategy(EvictionStrategy { max_safe_tokens: 4096 });
```

默认 8K 对 8K 上下文的模型比较安全。128K 模型可以设到 64K 或更高。具体值取决于你留多少输出空间。

## LLMEngineConfig

传给 `Agent::make()` 的枚举：

```rust
pub enum LLMEngineConfig {
    #[cfg(feature = "openai-api")]
    OpenAI(OpenAIEngineConfig),
    #[cfg(feature = "llama-cpp")]
    Llama(LlamaEngineConfig),
    Custom(Box<dyn LLMEngineTrait>),
}
```

### OpenAI 配置

```rust
OpenAIEngineConfig {
    api_key: String,
    base_url: String,    // "https://api.openai.com/v1"
    model_name: String,  // "gpt-4o"
    temp: f32,           // 0.0 - 2.0
    top_p: f32,          // 0.0 - 1.0
}
```

`base_url` 可以指向任何兼容 OpenAI 接口的端点（DeepSeek、Ollama 的 OpenAI 适配器等）。

### Llama.cpp 配置

```rust
LlamaEngineConfig {
    model_path: String,              // .gguf 模型文件路径
    mmproj_path: Option<String>,     // 外部视觉投影模型（如 mmproj-model-f16.gguf）
    integrated_vision: bool,         // 模型是否具有原生视觉能力
    max_tokens: i32,                 // 最大预测 token 数
    buffer_size: usize,              // batch 解码缓冲区大小
    use_gpu: bool,                   // 是否将层卸载到 GPU
    n_gpu_layers: u32,               // 卸载到 GPU 的层数
    n_ctx: u32,                      // 上下文窗口大小
    n_tokens: usize,                 // prompt 处理的 batch 大小
    n_seq_max: i32,                  // batch 中最大序列数
    penalty_last_n: i32,             // 惩罚时考虑的历史 token 数
    penalty_repeat: f32,             // 重复惩罚系数
    penalty_freq: f32,               // 频率惩罚系数
    penalty_present: f32,            // 存在惩罚系数
    temp: f32,                       // 温度 (0.0 – 2.0)
    top_p: f32,                      // 核采样阈值
    seed: u32,                       // 随机种子（确定性生成）
    min_keep: usize,                 // min-keep 采样边界
}
```

加载时会做校验——必填字段缺失或超范围会立即返回 `EngineError`，而不是推理到一半莫名其妙崩溃。

## 特性标志

```toml
[dependencies]
ambi = { version = "0.3", default-features = false, features = ["openai-api"] }
```

| 特性 | 启用什么 | 依赖 |
|------|---------|------|
| `openai-api` | OpenAI 兼容云后端 | `async-openai` |
| `llama-cpp` | 本地 llama.cpp 推理 | `llama-cpp-2`、`llama-cpp-sys-2` |
| `cuda` | CUDA 加速（隐含 llama-cpp） | + CUDA SDK |
| `vulkan` | Vulkan 加速 | + Vulkan SDK |
| `metal` | Apple Metal 加速 | + Metal framework |
| `rocm` | AMD ROCm 加速 | + ROCm |
| `macro` | `#[tool]` 和 `#[agent]` 属性宏（参见 [ambi-macros](/zh/extensions/ambi-macros)） | `ambi-macros` |
| `mtmd` | Llama 多模态支持 (VLM) | + `base64` |

编译时不能同时启用两个 GPU 后端——有一个 `compile_error!` 守卫。

## 运行时依赖

```toml
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time", "macros"] }
```

详见[原生平台](/zh/platform/native)。
