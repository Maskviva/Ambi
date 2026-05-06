// src/llm/providers/openai_api/stream.rs

use super::OpenAIEngine;
use crate::error::{AmbiError, Result};
use crate::types::LLMRequest;
use tokio::sync::mpsc::Sender;

#[cfg(not(target_arch = "wasm32"))]
use async_openai::types::chat::ChatCompletionMessageToolCallChunk;
#[cfg(not(target_arch = "wasm32"))]
use futures::StreamExt;
#[cfg(not(target_arch = "wasm32"))]
use log::debug;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

impl OpenAIEngine {
    /// Generates a streaming response from the OpenAI API.
    pub async fn generate_response_stream(
        &self,
        request: LLMRequest,
        tx: Sender<Result<String>>,
    ) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        {
            self.generate_response_stream_wasm(request, tx).await
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.generate_response_stream_native(request, tx).await
    }

    /// WASM streaming implementation using browser `fetch` + `ReadableStream`.
    #[cfg(target_arch = "wasm32")]
    async fn generate_response_stream_wasm(
        &self,
        request: LLMRequest,
        tx: Sender<Result<String>>,
    ) -> Result<()> {
        use js_sys::{Array as JsArray, ArrayBuffer, Reflect, Uint8Array, JSON as JsJson};
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{
            Headers, ReadableStream, ReadableStreamDefaultReader, Request, RequestInit, Response,
        };

        let model_name = self.cfg.model_name.clone();
        let api_request = self.get_request(model_name, request, true)?;
        let body_str = serde_json::to_string(&api_request)
            .map_err(|e| AmbiError::EngineError(format!("JSON serialize error: {}", e)))?;

        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_body(&JsValue::from_str(&body_str));

        let headers = Headers::new()
            .map_err(|e| AmbiError::EngineError(format!("Failed to create Headers: {:?}", e)))?;
        headers
            .set("Content-Type", "application/json")
            .map_err(|e| AmbiError::EngineError(format!("Failed to set Content-Type: {:?}", e)))?;
        headers
            .set("Authorization", &format!("Bearer {}", self.cfg.api_key))
            .map_err(|e| AmbiError::EngineError(format!("Failed to set Authorization: {:?}", e)))?;
        headers
            .set("Accept", "text/event-stream")
            .map_err(|e| AmbiError::EngineError(format!("Failed to set Accept: {:?}", e)))?;
        opts.set_headers(&headers);

        let base_url = self.cfg.base_url.trim_end_matches('/').to_string();
        let url = format!("{}/chat/completions", base_url);
        let js_req = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| AmbiError::EngineError(format!("Failed to create Request: {:?}", e)))?;

        let window = web_sys::window()
            .ok_or_else(|| AmbiError::EngineError("No window available (WASM)".into()))?;
        let response_val = JsFuture::from(window.fetch_with_request(&js_req))
            .await
            .map_err(|e| AmbiError::EngineError(format!("Fetch error: {:?}", e)))?;

        let response: Response = response_val.dyn_into().map_err(|_| {
            AmbiError::EngineError("Failed to cast fetch result to Response".into())
        })?;

        if !response.ok() {
            let status = response.status();
            let status_text = response.status_text();
            if !status_text.is_empty() {
                return Err(AmbiError::EngineError(format!(
                    "HTTP {}: {}",
                    status, status_text
                )));
            }
            return Err(AmbiError::EngineError(format!("HTTP error: {}", status)));
        }

        let body = response
            .body()
            .ok_or_else(|| AmbiError::EngineError("Response body is null/undefined".into()))?;
        let stream: ReadableStream = body
            .dyn_into()
            .map_err(|_| AmbiError::EngineError("Response body is not a ReadableStream".into()))?;

        let reader_val = stream.get_reader();
        let reader: ReadableStreamDefaultReader = reader_val.dyn_into().map_err(|_| {
            AmbiError::EngineError("Failed to get ReadableStreamDefaultReader".into())
        })?;

        let mut buffer = Vec::<u8>::new();
        loop {
            let chunk_val = JsFuture::from(reader.read()).await.map_err(|e| {
                AmbiError::EngineError(format!("ReadableStream read() error: {:?}", e))
            })?;
            let done = Reflect::get(&chunk_val, &JsValue::from_str("done"))
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let value = Reflect::get(&chunk_val, &JsValue::from_str("value"))
                .unwrap_or(JsValue::undefined());

            if done {
                break;
            }

            if value.is_undefined() || value.is_null() {
                continue;
            }

            let chunk_bytes = if let Some(uint8) = value.dyn_ref::<Uint8Array>() {
                uint8.to_vec()
            } else if let Some(buf) = value.dyn_ref::<ArrayBuffer>() {
                Uint8Array::new(buf).to_vec()
            } else {
                continue;
            };

            buffer.extend_from_slice(&chunk_bytes);

            while let Some(nl_pos) = buffer.iter().position(|&b| b == b'\n') {
                let line_bytes: Vec<u8> = buffer.drain(..=nl_pos).collect();
                let line_end = line_bytes.len().saturating_sub(1).saturating_sub(
                    if line_bytes.len() > 1 && line_bytes[line_bytes.len() - 2] == b'\r' {
                        1
                    } else {
                        0
                    },
                );

                let line = String::from_utf8_lossy(&line_bytes[..line_end]);
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Some(data) = trimmed.strip_prefix("data: ") {
                    let data = data.trim();
                    if data == "[DONE]" {
                        return Ok(());
                    }

                    if let Ok(json_val) = JsJson::parse(data) {
                        if let Ok(choices_val) =
                            Reflect::get(&json_val, &JsValue::from_str("choices"))
                        {
                            if JsArray::is_array(&choices_val) {
                                let arr = JsArray::from(&choices_val);
                                if arr.length() > 0 {
                                    let choice = arr.get(0);
                                    if let Ok(delta) =
                                        Reflect::get(&choice, &JsValue::from_str("delta"))
                                    {
                                        if let Ok(content_val) =
                                            Reflect::get(&delta, &JsValue::from_str("content"))
                                        {
                                            if let Some(text) = content_val.as_string() {
                                                if !text.is_empty()
                                                    && tx.send(Ok(text)).await.is_err()
                                                {
                                                    return Ok(());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Native-only streaming implementation using async-openai's create_stream.
    #[cfg(not(target_arch = "wasm32"))]
    async fn generate_response_stream_native(
        &self,
        request: LLMRequest,
        tx: Sender<Result<String>>,
    ) -> Result<()> {
        if let Some(msg) = request.history.last() {
            debug!("\n[OpenAI API] Request\n====================\n{}", msg);
        }

        let tool_tags = request.tool_tags.clone();
        let api_request = self.get_request(self.cfg.model_name.clone(), request, true)?;

        let mut stream = self
            .client
            .chat()
            .create_stream(api_request)
            .await
            .map_err(|e| AmbiError::EngineError(e.to_string()))?;

        let mut tool_calls_map: BTreeMap<u32, (String, String)> = BTreeMap::new();
        let mut tool_calls_started = false;

        while let Some(result) = stream.next().await {
            let response = match result {
                Ok(resp) => resp,
                Err(e) => {
                    let _ = tx
                        .send(Err(AmbiError::EngineError(format!(
                            "Stream interrupted: {}",
                            e
                        ))))
                        .await;
                    return Err(AmbiError::EngineError(e.to_string()));
                }
            };

            for choice in response.choices {
                if let Some(tool_calls) = choice.delta.tool_calls {
                    tool_calls_started = true;
                    Self::collect_tool_call_delta(&mut tool_calls_map, tool_calls);
                }

                if !tool_calls_started {
                    if let Some(content) = choice.delta.content {
                        if tx.send(Ok(content)).await.is_err() {
                            return Ok(());
                        }
                    }
                }
            }
        }

        if !tool_calls_map.is_empty() {
            let simulated = tool_calls_map
                .values()
                .map(|(name, args)| {
                    format!(
                        "{}{{\"name\":\"{}\",\"args\":{}}}{}",
                        tool_tags.0, name, args, tool_tags.1
                    )
                })
                .collect::<Vec<_>>()
                .join("");
            let _ = tx.send(Ok(simulated)).await;
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn collect_tool_call_delta(
        map: &mut BTreeMap<u32, (String, String)>,
        calls: Vec<ChatCompletionMessageToolCallChunk>,
    ) {
        let v = serde_json::to_value(&calls).unwrap_or_default();
        let arr = match v.as_array() {
            Some(a) => a,
            None => return,
        };

        for tc in arr {
            let idx = match tc.get("index").and_then(|i| i.as_u64()) {
                Some(i) => i as u32,
                None => continue,
            };

            let func = match tc.get("function") {
                Some(f) => f,
                None => continue,
            };

            let entry = map.entry(idx).or_default();

            if let Some(n) = func.get("name").and_then(|v| v.as_str()) {
                entry.0.push_str(n);
            }
            if let Some(a) = func.get("arguments").and_then(|v| v.as_str()) {
                entry.1.push_str(a);
            }
        }
    }
}
