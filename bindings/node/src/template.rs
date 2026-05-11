// bindings/node/src/template.rs

use ambi::types::ChatTemplate;
use napi_derive::napi;

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

impl From<&JsChatTemplate> for ChatTemplate {
    fn from(t: &JsChatTemplate) -> Self {
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

pub fn convert_chat_template(t: &ChatTemplate) -> JsChatTemplate {
    JsChatTemplate {
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

#[napi]
pub fn chatml_template() -> JsChatTemplate {
    convert_chat_template(&ChatTemplate::chatml())
}

#[napi]
pub fn llama3_template() -> JsChatTemplate {
    convert_chat_template(&ChatTemplate::llama3())
}

#[napi]
pub fn gemma_template() -> JsChatTemplate {
    convert_chat_template(&ChatTemplate::gemma())
}

#[napi]
pub fn phi3_template() -> JsChatTemplate {
    convert_chat_template(&ChatTemplate::phi3())
}

#[napi]
pub fn zephyr_template() -> JsChatTemplate {
    convert_chat_template(&ChatTemplate::zephyr())
}

#[napi]
pub fn deepseek_template() -> JsChatTemplate {
    convert_chat_template(&ChatTemplate::deepseek())
}

#[napi]
pub fn qwen_template() -> JsChatTemplate {
    convert_chat_template(&ChatTemplate::qwen())
}

#[napi]
pub fn mistral_template() -> JsChatTemplate {
    convert_chat_template(&ChatTemplate::mistral())
}

#[napi]
pub fn llama2_template() -> JsChatTemplate {
    convert_chat_template(&ChatTemplate::llama2())
}
