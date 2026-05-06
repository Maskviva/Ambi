# Native Platform (Linux / Windows / macOS)

## Runtime requirement

Ambi requires Tokio with multi-thread support. The minimal setup:

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time", "macros"] }
```

This is enforced by `Agent::make()` which calls `tokio::task::spawn_blocking()` to load the engine model without blocking the async runtime.

If you use `current_thread` runtime:

```rust
#[tokio::main(flavor = "current_thread")]
```

`Agent::make()` will panic because `spawn_blocking` requires a multi-thread runtime.

## `LLMEngineConfig::Custom` is different

`LLMEngineConfig::Custom` wraps a `Box<dyn LLMEngineTrait>` directly and does **not** call `spawn_blocking`. It works with any Tokio runtime:

```rust
use ambi::{Agent, LLMEngineConfig};

let agent = Agent::make(
    LLMEngineConfig::Custom(Box::new(MockEngine))
).await?; // no spawn_blocking
```

> **Note:** The old `Agent::with_custom_engine()` is deprecated since v0.3.3.
> Use `Agent::make(LLMEngineConfig::Custom(backend)).await` instead.

## GPU acceleration

For llama.cpp local inference, GPU offloading is configured at build time via Cargo features:

```toml
# CUDA (NVIDIA)
ambi = { version = "0.3", features = ["llama-cpp", "cuda"] }

# Vulkan (multi-vendor)
ambi = { version = "0.3", features = ["llama-cpp", "vulkan"] }

# Metal (Apple Silicon)
ambi = { version = "0.3", features = ["llama-cpp", "metal"] }

# ROCm (AMD)
ambi = { version = "0.3", features = ["llama-cpp", "rocm"] }
```

Only one GPU backend can be enabled at compile time. Enabling two or more causes a `compile_error!`.

## Building from source

```bash
# Cloud only (fastest compile)
cargo build --no-default-features --features openai-api

# Local with CUDA
cargo build --features "llama-cpp, cuda"
```

## Known platform differences

- **Windows**: llama.cpp CUDA builds require the CUDA SDK and MSVC build tools. Use the `x64-native-nvidia` toolchain if available.
- **macOS**: Metal acceleration works on Apple Silicon (M1+). Intel Macs fall back to CPU.
- **Linux**: CUDA requires `libcuda.so` and NVCC in PATH. Vulkan needs the Vulkan SDK.
