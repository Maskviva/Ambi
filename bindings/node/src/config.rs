// bindings/node/src/config.rs
//
//! Configuration objects the bindings ship over the wire — nothing more,
//! nothing less. Every struct here mirrors a Rust config type one-to-one,
//! so the JS side can tweak any knob the framework exposes.

use napi_derive::napi;

// ── EvictionStrategy ──

/// Controls how aggressively the Agent prunes old conversation history
/// when the token budget runs low. Uses a straightforward FIFO policy.
#[napi(object)]
pub struct JsEvictionStrategy {
    /// Messages above this token count get evicted, oldest first.
    pub max_safe_tokens: u32,
}

impl From<JsEvictionStrategy> for ambi::config::EvictionStrategy {
    fn from(js: JsEvictionStrategy) -> Self {
        ambi::config::EvictionStrategy {
            max_safe_tokens: js.max_safe_tokens as usize,
        }
    }
}

impl From<&ambi::config::EvictionStrategy> for JsEvictionStrategy {
    fn from(s: &ambi::config::EvictionStrategy) -> Self {
        Self {
            max_safe_tokens: s.max_safe_tokens as u32,
        }
    }
}

// ── ChatTemplateType ──

/// Chat template formats the framework knows out of the box.
/// Picking one is equivalent to setting `ChatTemplate::from(type)` in Rust.
#[napi]
pub enum JsChatTemplateType {
    Chatml,
    Llama3,
    Gemma,
    Phi3,
    Zephyr,
    Deepseek,
    Qwen,
    Mistral,
    Llama2,
}

impl From<JsChatTemplateType> for ambi::types::ChatTemplateType {
    fn from(ty: JsChatTemplateType) -> Self {
        match ty {
            JsChatTemplateType::Chatml => ambi::types::ChatTemplateType::Chatml,
            JsChatTemplateType::Llama3 => ambi::types::ChatTemplateType::Llama3,
            JsChatTemplateType::Gemma => ambi::types::ChatTemplateType::Gemma,
            JsChatTemplateType::Phi3 => ambi::types::ChatTemplateType::Phi3,
            JsChatTemplateType::Zephyr => ambi::types::ChatTemplateType::Zephyr,
            JsChatTemplateType::Deepseek => ambi::types::ChatTemplateType::Deepseek,
            JsChatTemplateType::Qwen => ambi::types::ChatTemplateType::Qwen,
            JsChatTemplateType::Mistral => ambi::types::ChatTemplateType::Mistral,
            JsChatTemplateType::Llama2 => ambi::types::ChatTemplateType::Llama2,
        }
    }
}

impl From<ambi::types::ChatTemplateType> for JsChatTemplateType {
    fn from(ty: ambi::types::ChatTemplateType) -> Self {
        match ty {
            ambi::types::ChatTemplateType::Chatml => JsChatTemplateType::Chatml,
            ambi::types::ChatTemplateType::Llama3 => JsChatTemplateType::Llama3,
            ambi::types::ChatTemplateType::Gemma => JsChatTemplateType::Gemma,
            ambi::types::ChatTemplateType::Phi3 => JsChatTemplateType::Phi3,
            ambi::types::ChatTemplateType::Zephyr => JsChatTemplateType::Zephyr,
            ambi::types::ChatTemplateType::Deepseek => JsChatTemplateType::Deepseek,
            ambi::types::ChatTemplateType::Qwen => JsChatTemplateType::Qwen,
            ambi::types::ChatTemplateType::Mistral => JsChatTemplateType::Mistral,
            ambi::types::ChatTemplateType::Llama2 => JsChatTemplateType::Llama2,
        }
    }
}

// ── ChatTemplate ──

/// The full set of delimiters that stitch a conversation into a single
/// prompt string. Every prefix and suffix the template engine touches
/// is exposed here, so you can build a custom format from scratch
/// instead of hunting through source code.
#[napi(object)]
pub struct JsChatTemplate {
    pub system_prefix: String,
    pub system_suffix: String,
    pub user_prefix: String,
    pub user_suffix: String,
    pub assistant_prefix: String,
    pub assistant_suffix: String,
    pub think_prefix: String,
    pub think_suffix: String,
    pub tool_prefix: String,
    pub tool_suffix: String,
    pub tool_id_prefix: String,
    pub tool_id_suffix: String,
    pub media_placeholder: String,
}

impl From<JsChatTemplate> for ambi::types::ChatTemplate {
    fn from(js: JsChatTemplate) -> Self {
        ambi::types::ChatTemplate {
            system_prefix: js.system_prefix,
            system_suffix: js.system_suffix,
            user_prefix: js.user_prefix,
            user_suffix: js.user_suffix,
            assistant_prefix: js.assistant_prefix,
            assistant_suffix: js.assistant_suffix,
            think_prefix: js.think_prefix,
            think_suffix: js.think_suffix,
            tool_prefix: js.tool_prefix,
            tool_suffix: js.tool_suffix,
            tool_id_prefix: js.tool_id_prefix,
            tool_id_suffix: js.tool_id_suffix,
            media_placeholder: js.media_placeholder,
        }
    }
}

impl From<&ambi::types::ChatTemplate> for JsChatTemplate {
    fn from(t: &ambi::types::ChatTemplate) -> Self {
        Self {
            system_prefix: t.system_prefix.clone(),
            system_suffix: t.system_suffix.clone(),
            user_prefix: t.user_prefix.clone(),
            user_suffix: t.user_suffix.clone(),
            assistant_prefix: t.assistant_prefix.clone(),
            assistant_suffix: t.assistant_suffix.clone(),
            think_prefix: t.think_prefix.clone(),
            think_suffix: t.think_suffix.clone(),
            tool_prefix: t.tool_prefix.clone(),
            tool_suffix: t.tool_suffix.clone(),
            tool_id_prefix: t.tool_id_prefix.clone(),
            tool_id_suffix: t.tool_id_suffix.clone(),
            media_placeholder: t.media_placeholder.clone(),
        }
    }
}

// ── OpenAIEngineConfig ──

/// Connection parameters for any OpenAI-compatible API.
/// Works with OpenAI, DeepSeek, Groq, vLLM, Ollama — anything that
/// speaks the Chat Completions wire format.
#[napi(object)]
pub struct JsOpenAIConfig {
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
    /// Sampling temperature, 0.0 – 2.0.
    pub temp: f64,
    /// Top-p nucleus sampling threshold.
    pub top_p: f64,
}
