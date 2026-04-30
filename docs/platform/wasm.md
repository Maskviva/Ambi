# WebAssembly (WASM)

Ambi compiles to WASM32 and runs in browsers. This is a first-class target, not an afterthought.

## Limitations compared to native

| Feature | Native | WASM |
|---------|--------|------|
| llama.cpp inference | Yes | **No** (compile-time blocked) |
| OpenAI API | Yes | Yes (browser fetch) |
| Custom engine | Yes | Yes |
| `spawn_blocking` | Thread pool | Inline execution |
| `Send + Sync` bounds | Enforced | Relaxed (single-threaded) |
| GPU acceleration | Yes | No |

The `llama-cpp` feature is blocked at compile time for WASM:

```rust
#[cfg(all(target_arch = "wasm32", feature = "llama-cpp"))]
compile_error!("llama-cpp not supported on wasm32");
```

Only `openai-api` or custom engines work on WASM.

## Building for WASM

```bash
cargo build --target wasm32-unknown-unknown --no-default-features --features openai-api
```

Or use `wasm-pack` for a browser-ready package:

```bash
wasm-pack build --target web --no-default-features --features openai-api
```

## Runtime polyfills

The `runtime` module replaces Tokio-specific calls with WASM-compatible alternatives:

- **`spawn()`** → `wasm_bindgen_futures::spawn_local()`
- **`spawn_blocking()`** → direct synchronous execution (single-threaded)
- **`sleep()`** → `gloo_timers::future::sleep()`
- **`timeout()`** → future race against a timer
- **`SendSync` trait** → empty marker (no-op in single-threaded context)

You don't need to change any code – the polyfills are applied automatically based on `#[cfg(target_arch = "wasm32")]`.

## Cargo.toml for WASM

```toml
[dependencies]
ambi = { version = "0.3", default-features = false, features = ["openai-api"] }
tokio = { version = "1", features = ["sync", "macros"] }    # no rt-multi-thread
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
```

Note: `rt-multi-thread` is not needed (and won't compile) for WASM.

## Example

See `examples/webassembly.rs` for a complete browser-ready setup.
