# 原生平台 (Linux / Windows / macOS)

## 运行时要求

Ambi 需要 Tokio 的多线程支持。最小配置：

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time", "macros"] }
```

这是 `Agent::make()` 要求的，它内部调用 `tokio::task::spawn_blocking()` 来加载模型，不会阻塞异步运行时。

如果用了 `current_thread` 运行时：

```rust
#[tokio::main(flavor = "current_thread")]
```

`Agent::make()` 会 panic，因为 `spawn_blocking` 依赖多线程运行时。

## LLMEngineConfig::Custom 不一样

`LLMEngineConfig::Custom` 直接包装 `Box<dyn LLMEngineTrait>`，**不会**调 `spawn_blocking`。它在任何 Tokio 运行时下都能工作：

```rust
use ambi::{Agent, LLMEngineConfig};

let agent = Agent::make(
    LLMEngineConfig::Custom(Box::new(MockEngine))
).await?; // 不需要 spawn_blocking
```

> **注意：** 旧的 `Agent::with_custom_engine()` 自 v0.3.3 起已废弃。
> 请使用 `Agent::make(LLMEngineConfig::Custom(backend)).await` 替代。

## GPU 加速

llama.cpp 本地推理的 GPU 卸载在构建时通过 Cargo 特性配置：

```toml
# CUDA (NVIDIA)
ambi = { version = "0.3", features = ["llama-cpp", "cuda"] }

# Vulkan（多厂商）
ambi = { version = "0.3", features = ["llama-cpp", "vulkan"] }

# Metal (Apple Silicon)
ambi = { version = "0.3", features = ["llama-cpp", "metal"] }

# ROCm (AMD)
ambi = { version = "0.3", features = ["llama-cpp", "rocm"] }
```

编译时只能启用一个 GPU 后端，多个会导致 `compile_error!`。

## 从源码构建

```bash
# 只用云后端（编译最快）
cargo build --no-default-features --features openai-api

# 本地 + CUDA
cargo build --features "llama-cpp, cuda"
```

## 已知平台差异

- **Windows**：llama.cpp CUDA 构建需要 CUDA SDK 和 MSVC 构建工具。
- **macOS**：Metal 加速只在 Apple Silicon（M1+）上工作。Intel Mac 回退到 CPU。
- **Linux**：CUDA 需要 `libcuda.so` 和 NVCC。Vulkan 需要 Vulkan SDK。
