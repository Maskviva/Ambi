use crate::config::JsLLMEngineConfig;
use crate::template::{JsChatTemplate, JsChatTemplateType};
use crate::tool::JsToolBridge;
use ambi::config::EvictionStrategy;
use ambi::types::ChatTemplate;
use ambi::Agent;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;
use serde_json::Value;

#[napi(js_name = "Agent")]
#[derive(Clone)]
pub struct JsAgent {
    pub(crate) inner: Agent,
}

#[napi(object)]
pub struct JsEvictionStrategy {
    pub max_safe_tokens: u32,
}

#[napi]
impl JsAgent {
    #[napi]
    pub async fn make(config: &JsLLMEngineConfig) -> Result<JsAgent> {
        let engine_cfg = {
            let mut lock = config.inner.lock().map_err(|_| {
                Error::from_reason("Internal concurrency error: Failed to lock config")
            })?;
            lock.take().ok_or_else(|| {
                Error::from_reason("LLMEngineConfig can only be used once to create an Agent.")
            })?
        };
        let agent = Agent::make(engine_cfg)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(JsAgent { inner: agent })
    }

    #[napi]
    pub fn preamble(&mut self, text: String) -> Self {
        self.inner = self.inner.clone().preamble(&text);
        self.clone()
    }

    #[napi]
    pub fn template(&mut self, template_type: JsChatTemplateType) -> Self {
        use ambi::types::ChatTemplateType::*;
        let ct = match template_type {
            JsChatTemplateType::Chatml => Chatml,
            JsChatTemplateType::Llama3 => Llama3,
            JsChatTemplateType::Gemma => Gemma,
            JsChatTemplateType::Phi3 => Phi3,
            JsChatTemplateType::Zephyr => Zephyr,
            JsChatTemplateType::Deepseek => Deepseek,
            JsChatTemplateType::Qwen => Qwen,
            JsChatTemplateType::Mistral => Mistral,
            JsChatTemplateType::Llama2 => Llama2,
        };
        self.inner = self.inner.clone().template(ct);
        self.clone()
    }

    #[napi]
    pub fn custom_template(&mut self, template: JsChatTemplate) -> Self {
        self.inner = self.inner.clone().template(ChatTemplate::from(&template));
        self.clone()
    }

    #[napi]
    pub fn with_eviction_strategy(&mut self, strategy: JsEvictionStrategy) -> Self {
        let s = EvictionStrategy {
            max_safe_tokens: strategy.max_safe_tokens as usize,
        };
        self.inner = self.inner.clone().with_eviction_strategy(s);
        self.clone()
    }

    #[napi]
    pub fn max_iterations(&mut self, n: u32) -> Self {
        self.inner = self.inner.clone().max_iterations(n as usize);
        self.clone()
    }

    #[napi]
    pub fn with_standard_formatting(&mut self) -> Self {
        self.inner = self.inner.clone().with_standard_formatting();
        self.clone()
    }

    #[napi(
        ts_args_type = "name: string, description: string, parameters_json_str: string, callback: (_err: Error | null, argsJson: string) => string, timeoutSecs?: number, maxRetries?: number, isIdempotent?: boolean"
    )]
    #[allow(clippy::too_many_arguments)]
    pub fn tool(
        &mut self,
        name: String,
        description: String,
        parameters_json_str: String,
        callback: Function,
        timeout_secs: Option<u32>,
        max_retries: Option<u32>,
        is_idempotent: Option<bool>,
    ) -> Result<Self> {
        let val = callback.value();
        let tsfn: ThreadsafeFunction<String, String> =
            unsafe { FromNapiValue::from_napi_value(val.env, val.value)? };

        let parameters: Value = serde_json::from_str(&parameters_json_str)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let bridge = JsToolBridge {
            name,
            description,
            parameters,
            timeout_secs: timeout_secs.map(|v| v as u64),
            max_retries: max_retries.map(|v| v as usize),
            is_idempotent: is_idempotent.unwrap_or(true),
            callback: tsfn,
        };

        self.inner = self
            .inner
            .clone()
            .tool(bridge)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(self.clone())
    }

    #[napi]
    pub fn with_tool_tags(&mut self, start_tag: String, end_tag: String) -> Self {
        self.inner = self.inner.clone().with_tool_tags(&start_tag, &end_tag);
        self.clone()
    }

    #[napi]
    pub fn count_tokens(&self, text: String) -> Result<u32> {
        let engine = self.inner.get_llama_engine();
        engine
            .count_tokens(&text)
            .map(|n| n as u32)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub fn on_evict(&mut self, callback: Function) -> Result<Self> {
        let val = callback.value();
        let tsfn: ThreadsafeFunction<String, String> =
            unsafe { FromNapiValue::from_napi_value(val.env, val.value)? };
        self.inner = self.inner.clone().on_evict(move |_state, messages| {
            let json = serde_json::to_string(&messages).unwrap_or_default();
            let _ = tsfn.call(
                Ok(json),
                napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
            );
        });
        Ok(self.clone())
    }
}
