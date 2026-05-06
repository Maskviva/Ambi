# WebAssembly (WASM)

Ambi 可以编译到 WASM32 在浏览器里运行。这是一等公民目标，不是事后才想的事。

## 与原生平台的差异

| 特性 | 原生 | WASM |
|------|------|------|
| llama.cpp 推理 | 支持 | **不支持**（编译时阻止） |
| OpenAI API | 支持 | 支持（浏览器 fetch） |
| **流式 API** | 支持（tokio） | **支持**（原生 `fetch` + `ReadableStream`） |
| 自定义引擎 | 支持 | 支持 |
| `spawn_blocking` | 线程池 | 直接执行 |
| `Send + Sync` 约束 | 强制执行 | 放宽（单线程） |
| GPU 加速 | 支持 | 不支持 |

`llama-cpp` 特性在 WASM 下被编译时阻止：

```rust
#[cfg(all(target_arch = "wasm32", feature = "llama-cpp"))]
compile_error!("llama-cpp not supported on wasm32");
```

WASM 只能用 `openai-api` 或自定义引擎。

## 构建 WASM

```bash
cargo build --target wasm32-unknown-unknown --no-default-features --features openai-api
```

或者用 `wasm-pack` 打出浏览器可用的包：

```bash
wasm-pack build --target web --no-default-features --features openai-api
```

## 运行时填充

`runtime` 模块自动把 Tokio 特有的调用替换成 WASM 兼容版本：

- **`spawn()`** → `wasm_bindgen_futures::spawn_local()`
- **`spawn_blocking()`** → 直接同步执行（单线程）
- **`sleep()`** → `gloo_timers::future::sleep()`
- **`timeout()`** → Future 和计时器竞速
- **`SendSync` trait** → 空标记（单线程上下文下无意义）

不需要改任何代码——填充层基于 `#[cfg(target_arch = "wasm32")]` 自动应用。

## WASM 的 Cargo.toml

```toml
[dependencies]
ambi = { version = "0.3", default-features = false, features = ["openai-api"] }
tokio = { version = "1", features = ["sync", "macros"] }    # 不需要 rt-multi-thread
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
```

注意：`rt-multi-thread` 在 WASM 下不需要（也编译不过）。

## 浏览器中的流式响应

WASM 平台的 OpenAI 提供者使用原生的 `fetch` 和 `ReadableStream` API 实现真正的流式传输。
相同的 `chat_stream()` API 在浏览器中完全一致：

```rust
use futures::StreamExt;

let mut stream = runner.chat_stream(&agent, &state, "讲个故事").await?;
while let Some(chunk) = stream.next().await {
    if let Ok(text) = chunk {
        // 追加到 DOM
    }
}
```

不需要特殊的 WASM polyfill——`runtime` 模块自动将 Tokio 内部实现替换为 WASM 兼容的替代方案。

## 示例

参考 [`examples/webAssembly`](https://github.com/maskviva/ambi/tree/main/examples/webAssembly)
查看完整的浏览器就绪示例，包含展示实时流式文本生成的 UI 切换演示。
