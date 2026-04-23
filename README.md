# Ambi 🦀

> [中文](./README_zh.md)

A flexible, highly customizable AI Agent framework built entirely in Rust.

## Philosophy

Ambi was born out of frustration with overly complex agent frameworks that force you into rigid architectures and endless configuration.

- **Minimal Boilerplate**: Get a working agent in 5 lines of code. No magic, no hidden state.
- **Trait-First Design**: Every component is replaceable. Bring your own LLM backend, tool parser, or execution pipeline.
- **We build the wheels, you build the car**: We handle the low-level robustness (OOM protection, retries, context management) so you can focus on your application logic.
- **Rust Native**: Zero-cost abstractions, memory safety, and native performance for production-grade agents.

## Key Features

### Dual-Engine Architecture
- **Local Inference**: Powered by `llama.cpp` with full hardware acceleration (CUDA, Vulkan, Metal)
- **Cloud APIs**: 100% compatible with OpenAI-spec endpoints (DeepSeek, SiliconFlow, Groq, and more)
- Seamless switching between engines without changing your agent code

### Advanced Tool System
- Parallel multi-tool calling in a single response
- Per-tool configuration: independent `timeout_secs` and `max_retries`
- Built-in tool idempotency and safety guards
- Automatic JSON schema generation from Rust structs

### Intelligent Context Management
- Safe context eviction algorithm that preserves conversation logic
- Evicts at natural message boundaries to prevent token overflow
- Configurable context window limits

### Developer Experience
- Native support for all major chat templates (ChatML, Llama3, Gemma, DeepSeek)
- Streaming reasoning with automatic `` tag formatting
- Comprehensive error handling with meaningful error messages
- Minimal dependencies and fast compilation times

### Production Readiness
- OOM protection for local inference
- Graceful shutdown and resource cleanup
- Extensive test coverage
- Apache-2.0 licensed

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ambi = "0.1.7"
```

For cloud-only usage (faster compilation, no llama.cpp dependency):

```toml
ambi = { version = "0.1.7", default-features = false, features = ["openai-api"] }
```

## Usage

The source code is the best documentation. See the `/examples` directory for complete working examples:

- Basic chat agent
- Custom tool definition
- Local inference with GPU acceleration
- Streaming responses
- Multi-tool parallel execution

## License

Licensed under the Apache-2.0 License.