# Multimodal Input

Ambi supports image inputs if the engine backend handles them. This works with OpenAI's vision models (gpt-4o, gpt-4-vision) and local VLM models through llama.cpp.

## Sending images

Use `ContentPart::Image` with a base64-encoded image string:

```rust
use ambi::ContentPart;
use ambi::types::Message;

let parts = vec![
    ContentPart::Text { text: "What's in this image?".into() },
    ContentPart::Image { base64: image_base64_string },
];

let reply = runner.execute(&agent, &state, parts).await?;
```

Or use the convenience method:

```rust
let msg = Message::user_multimodal("Describe this", &image_base64);
```

### URL support

The `base64` field in `ContentPart::Image` accepts either a base64 data string or an HTTP URL. For OpenAI backends, sending a URL directly is more efficient:

```rust
ContentPart::Image { base64: "https://example.com/photo.jpg".into() }
```

## Fail-fast

If you send an image to an engine that doesn't support multimodal, you get an `EngineError` immediately:

```
Security Check Failed: The current LLM engine does not support multimodal (image) inputs.
```

This check runs before any tokens are sent, so you don't waste API calls.

## Engine support

| Engine | Multimodal | Notes |
|--------|-----------|-------|
| OpenAI (`gpt-4o`, `gpt-4-vision`) | Yes | URL or base64 |
| Llama.cpp vision models | Yes (with `mtmd` feature) | Qwen2-VL, LLaVA |
| Custom engine | Depends on `supports_multimodal()` | Trait method returns false by default |
