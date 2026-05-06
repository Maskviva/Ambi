# WebAssembly (WASM)

Ambi compiles to WASM32 and runs in browsers. This is a first-class target, not an afterthought.

## Limitations compared to native

| Feature | Native | WASM |
|---------|--------|------|
| llama.cpp inference | Yes | **No** (compile-time blocked) |
| OpenAI API | Yes | Yes (browser fetch) |
| **Streaming API** | Yes (tokio) | **Yes** (native `fetch` + `ReadableStream`) |
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

## Streaming in the browser

The OpenAI provider for WASM uses native `fetch` and `ReadableStream` APIs for true streaming.
The same `chat_stream()` API works identically in the browser:

```rust
use futures::StreamExt;

let mut stream = runner.chat_stream(&agent, &state, "Tell me a story").await?;
while let Some(chunk) = stream.next().await {
    if let Ok(text) = chunk {
        // append to DOM
    }
}
```

No special WASM polyfills are needed – the `runtime` module automatically swaps Tokio
internals for WASM-compatible alternatives.

## Example

See [`examples/webAssembly`](https://github.com/maskviva/ambi/tree/main/examples/webAssembly)
for a complete browser-ready setup with a UI toggle demoing real-time streaming text generation.
