# WebAssembly: share the API, respect the platform

Ambi has WASM-specific async adapters for browser deployments using OpenAI-compatible APIs or custom engines. `llama-cpp` is **compile-time rejected** on `wasm32`.

## Minimal dependencies

```toml
[dependencies]
ambi = { version = "0.4.1", default-features = false, features = ["openai-api"] }
tokio = { version = "1", features = ["sync", "macros"] }
```

Build a crate for `wasm32-unknown-unknown` with its usual WASM toolchain and any UI/build dependencies your application needs. The repository's [`examples/webAssembly`](https://github.com/Maskviva/Ambi/tree/main/examples/webAssembly) contains an application-level integration example.

## What changes from native?

| Capability | Native | Browser WASM |
| --- | --- | --- |
| OpenAI-compatible HTTP | Yes | Via browser fetch APIs |
| Streaming | Yes | Via ReadableStream adapter |
| Local llama.cpp | Yes | **No** |
| Native GPU backends | Feature-dependent | **No** |
| Custom engines | Supported | Supported, subject to WASM execution constraints |
| Async spawning | Tokio | WASM-local adapters |

The browser's network and security model still applies: CORS and **exposed API secrets** are application concerns. Do not embed privileged cloud API keys in public client bundles.

Portable public APIs do not imply identical resource or thread semantics on both targets. See [Architecture](/advanced/architecture) and [Configuration](/guide/configuration).
