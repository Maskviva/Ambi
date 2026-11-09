# Native targets: Linux, Windows, macOS

Ambi's native path supports OpenAI-compatible APIs, local llama.cpp inference and custom engines. The platform-specific choice should stay outside your higher-level `Pipeline` whenever possible.

## Runtime and build

```toml
[dependencies]
ambi = "0.4.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

The recommended native setup uses Tokio's multi-thread runtime because `Agent::make()` routes engine initialization through the crate's blocking-task adapter. Do not rely on a `current_thread` setup without testing your exact backend and executor.

To build local inference:

```toml
ambi = { version = "0.4.1", default-features = false, features = ["llama-cpp"] }
```

GPU-specific features are `cuda`, `vulkan`, `metal`, `rocm`; **do not enable more than one**. Hardware tools, SDKs and driver requirements depend on the target platform. `dynamic-link` is available where appropriate.

Use the conservative [`LlamaEngineConfig::builder()`](/guide/configuration) defaults and override context, batch size, model path and GPU settings as needed. Local inference runs behind a worker queue; session switching may require KV re-prefill.

## Smoke tests

```bash
cargo check --no-default-features --features openai-api
cargo test --no-default-features --features openai-api
cargo test --no-default-features --features "openai-api,sub-agent" --test leaf_subagent_test
```

The last command exercises the optional Leaf integration tests and requires a compatible environment. It is not included in every project's default test invocation automatically.

See [Getting started](/guide/getting-started) and [Architecture](/advanced/architecture).
