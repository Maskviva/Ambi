# Multimodal input: capability belongs to the engine

Ambi's request format can carry text and image content together. Whether an image can be processed depends on the **selected engine**, its model and the compiled features.

## Send image content

```rust
use ambi::agent::pipeline::Pipeline;
use ambi::ContentPart;

let parts = vec![
    ContentPart::Text { text: "Describe this image".into() },
    ContentPart::Image { base64: image_data },
];
let answer = runner.execute(&agent, &state, parts).await?;
```

`image_data` must be supplied by the application. The `base64` field supports a Base64/data URL, and some backends also accept an HTTP image URL. Do not assume the same image representation is supported by every engine.

## OpenAI-compatible API

`OpenAIEngineConfig::create()` sets `supports_vision` for recognized model-name prefixes. When a compatible endpoint supports vision under an unrecognized name, opt in using `.with_vision(true)`. This flag **declares** support; it does not inspect the remote model. Inputs with images are rejected up front if the engine reports `supports_multimodal() == false`.

## Local llama.cpp

Image processing depends on a compatible model, the `llama-cpp` backend and the `mtmd` feature; some architectures require a matching external `mmproj` file while others have integrated support. Simply enabling a feature does not turn a text-only GGUF into a vision model.

```toml
ambi = { version = "0.4.1", default-features = false, features = ["llama-cpp", "mtmd"] }
```

Image inputs are counted and formatted differently across providers. Do not treat text-only eviction estimates as a precise multimodal context budget. See [Configuration](/guide/configuration) and [Context eviction](/advanced/context-eviction).
