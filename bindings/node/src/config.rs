// bindings/node/src/config.rs
use crate::engine::JsEngineBridge;
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::LLMEngineConfig;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;
use std::sync::Mutex;

#[napi(object)]
pub struct JsOpenAIEngineConfig {
    pub api_key: String,
    pub model_name: String,
    pub base_url: Option<String>,
    pub temp: Option<f64>,
    pub top_p: Option<f64>,
}

#[napi(js_name = "LLMEngineConfig")]
pub struct JsLLMEngineConfig {
    pub(crate) inner: Mutex<Option<LLMEngineConfig>>,
}

#[napi]
impl JsLLMEngineConfig {
    #[napi(factory)]
    pub fn openai(config: JsOpenAIEngineConfig) -> Self {
        let cfg = OpenAIEngineConfig {
            api_key: config.api_key,
            base_url: config
                .base_url
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model_name: config.model_name,
            temp: config.temp.unwrap_or(0.0) as f32,
            top_p: config.top_p.unwrap_or(0.0) as f32,
        };
        Self {
            inner: Mutex::new(Some(LLMEngineConfig::OpenAI(cfg))),
        }
    }

    #[napi(factory, ts_args_type = "chatHandler: (_err: Error | null, argsJson: string) => void, supportsMultimodal?: boolean, chatStreamHandler?: (_err: Error | null, argsJson: string) => void")]
    pub fn custom(
        chat_handler: Function,
        supports_multimodal: Option<bool>,
        chat_stream_handler: Option<Function>,
    ) -> Result<Self> {
        let val = chat_handler.value();
        let tsfn: ThreadsafeFunction<String> =
            unsafe { FromNapiValue::from_napi_value(val.env, val.value)? };

        let stream_tsfn = if let Some(sh) = chat_stream_handler {
            let val = sh.value();
            Some(unsafe {
                FromNapiValue::from_napi_value(val.env, val.value)?
            })
        } else {
            None
        };

        let bridge = JsEngineBridge {
            chat_fn: tsfn,
            chat_stream_fn: stream_tsfn,
            supports_vision: supports_multimodal.unwrap_or(false),
        };
        Ok(Self {
            inner: Mutex::new(Some(LLMEngineConfig::Custom(Box::new(bridge)))),
        })
    }
}
