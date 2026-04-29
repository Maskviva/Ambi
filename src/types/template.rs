// src/types/template.rs

//! Templating configurations defining how different LLMs format conversations.

use serde::{Deserialize, Serialize};

/// Supported default chat template formats matching modern LLM specifications.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChatTemplateType {
    /// ChatML format (used by OpenAI, Qwen, etc.)
    #[default]
    Chatml,
    /// Llama 3 instructional format.
    Llama3,
    /// Google Gemma instructional format.
    Gemma,
    /// Microsoft Phi-3 instructional format.
    Phi3,
    /// Zephyr format (HuggingFace H4).
    Zephyr,
    /// DeepSeek instructional and reasoning format.
    Deepseek,
    /// Qwen instructional format.
    Qwen,
    /// Mistral standard instructional format.
    Mistral,
    /// Legacy Llama 2 format.
    Llama2,
}

impl From<ChatTemplateType> for ChatTemplate {
    fn from(template_type: ChatTemplateType) -> Self {
        template_type.as_template()
    }
}

impl From<&ChatTemplateType> for ChatTemplate {
    fn from(template_type: &ChatTemplateType) -> Self {
        template_type.as_template()
    }
}

impl ChatTemplateType {
    /// Converts the enum variant into a fully hydrated `ChatTemplate` instance.
    pub fn as_template(&self) -> ChatTemplate {
        match self {
            ChatTemplateType::Chatml => ChatTemplate::chatml(),
            ChatTemplateType::Llama3 => ChatTemplate::llama3(),
            ChatTemplateType::Gemma => ChatTemplate::gemma(),
            ChatTemplateType::Phi3 => ChatTemplate::phi3(),
            ChatTemplateType::Zephyr => ChatTemplate::zephyr(),
            ChatTemplateType::Deepseek => ChatTemplate::deepseek(),
            ChatTemplateType::Qwen => ChatTemplate::qwen(),
            ChatTemplateType::Mistral => ChatTemplate::mistral(),
            ChatTemplateType::Llama2 => ChatTemplate::llama2(),
        }
    }
}

/// Defines the prefix and suffix tags used to stitch dialogue contexts into a raw string prompt.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatTemplate {
    /// The prefix wrapping system instructions.
    pub system_prefix: String,
    /// The suffix closing system instructions.
    pub system_suffix: String,

    /// The prefix wrapping user queries.
    pub user_prefix: String,
    /// The suffix closing user queries.
    pub user_suffix: String,

    /// The prefix wrapping the assistant's responses.
    pub assistant_prefix: String,
    /// The suffix closing the assistant's responses.
    pub assistant_suffix: String,

    /// The prefix indicating the start of a model's internal reasoning block (e.g., `<think>`).
    pub think_prefix: String,
    /// The suffix indicating the end of a model's internal reasoning block.
    pub think_suffix: String,

    /// The prefix wrapping executed tool return values.
    pub tool_prefix: String,
    /// The suffix closing executed tool return values.
    pub tool_suffix: String,

    /// The prefix wrapping the tool execution ID.
    pub tool_id_prefix: String,
    /// The suffix closing the tool execution ID.
    pub tool_id_suffix: String,

    /// The explicit token used by vision-language models to mark image positions.
    pub media_placeholder: String,
}

impl ChatTemplate {
    /// Generates a ChatML compliant template.
    pub fn chatml() -> Self {
        Self {
            system_prefix: "<|im_start|>system\n".to_string(),
            system_suffix: "<|im_end|>\n".to_string(),
            user_prefix: "<|im_start|>user\n".to_string(),
            user_suffix: "<|im_end|>\n".to_string(),
            assistant_prefix: "<|im_start|>assistant\n".to_string(),
            assistant_suffix: "<|im_end|>\n".to_string(),
            think_prefix: "<think>\n".to_string(),
            think_suffix: "\n</think>\n".to_string(),
            tool_prefix: "<|im_start|>tool\n".to_string(),
            tool_suffix: "<|im_end|>\n".to_string(),
            tool_id_prefix: String::new(),
            tool_id_suffix: String::new(),
            media_placeholder: "<|image_pad|>".to_string(),
        }
    }

    /// Generates a Llama 3 compliant template.
    pub fn llama3() -> Self {
        Self {
            system_prefix: "<|start_header_id|>system<|end_header_id|>\n\n".to_string(),
            system_suffix: "<|eot_id|>\n".to_string(),
            user_prefix: "<|start_header_id|>user<|end_header_id|>\n\n".to_string(),
            user_suffix: "<|eot_id|>\n".to_string(),
            assistant_prefix: "<|start_header_id|>assistant<|end_header_id|>\n\n".to_string(),
            assistant_suffix: "<|eot_id|>\n".to_string(),
            think_prefix: "<think>\n".to_string(),
            think_suffix: "\n</think>\n".to_string(),
            tool_prefix: "<|start_header_id|>tool<|end_header_id|>\n\n".to_string(),
            tool_suffix: "<|eot_id|>\n".to_string(),
            tool_id_prefix: String::new(),
            tool_id_suffix: String::new(),
            media_placeholder: "<|image|>".to_string(),
        }
    }

    /// Generates a Gemma compliant template.
    pub fn gemma() -> Self {
        Self {
            system_prefix: "<start_of_turn>system\n".to_string(),
            system_suffix: "<end_of_turn>\n".to_string(),
            user_prefix: "<start_of_turn>user\n".to_string(),
            user_suffix: "<end_of_turn>\n".to_string(),
            assistant_prefix: "<start_of_turn>model\n".to_string(),
            assistant_suffix: "<end_of_turn>\n".to_string(),
            think_prefix: "<think>\n".to_string(),
            think_suffix: "\n</think>\n".to_string(),
            tool_prefix: "<start_of_turn>tool\n".to_string(),
            tool_suffix: "<end_of_turn>\n".to_string(),
            tool_id_prefix: String::new(),
            tool_id_suffix: String::new(),
            media_placeholder: "<image>".to_string(),
        }
    }

    /// Generates a Phi-3 compliant template.
    pub fn phi3() -> Self {
        Self {
            system_prefix: "<|system|>\n".to_string(),
            system_suffix: "<|end|>\n".to_string(),
            user_prefix: "<|user|>\n".to_string(),
            user_suffix: "<|end|>\n".to_string(),
            assistant_prefix: "<|assistant|>\n".to_string(),
            assistant_suffix: "<|end|>\n".to_string(),
            think_prefix: "<think>\n".to_string(),
            think_suffix: "\n</think>\n".to_string(),
            tool_prefix: "<|tool|>\n".to_string(),
            tool_suffix: "<|end|>\n".to_string(),
            tool_id_prefix: String::new(),
            tool_id_suffix: String::new(),
            media_placeholder: "<|image_1|>".to_string(),
        }
    }

    /// Generates a Zephyr compliant template.
    pub fn zephyr() -> Self {
        Self {
            system_prefix: "<|system|>\n".to_string(),
            system_suffix: "</s>\n".to_string(),
            user_prefix: "<|user|>\n".to_string(),
            user_suffix: "</s>\n".to_string(),
            assistant_prefix: "<|assistant|>\n".to_string(),
            assistant_suffix: "</s>\n".to_string(),
            think_prefix: "<think>\n".to_string(),
            think_suffix: "\n</think>\n".to_string(),
            tool_prefix: "<|tool|>\n".to_string(),
            tool_suffix: "</s>\n".to_string(),
            tool_id_prefix: String::new(),
            tool_id_suffix: String::new(),
            media_placeholder: "<image>".to_string(),
        }
    }

    /// Generates a DeepSeek compliant template.
    pub fn deepseek() -> Self {
        Self {
            system_prefix: "<｜begin of sentence｜>".to_string(),
            system_suffix: "\n\n".to_string(),
            user_prefix: "<｜User｜>".to_string(),
            user_suffix: "".to_string(),
            assistant_prefix: "<｜Assistant｜>".to_string(),
            assistant_suffix: "<｜end of sentence｜>".to_string(),
            think_prefix: "<think>\n".to_string(),
            think_suffix: "\n</think>\n".to_string(),
            tool_prefix: "<｜tool output｜>\n".to_string(),
            tool_suffix: "\n".to_string(),
            tool_id_prefix: String::new(),
            tool_id_suffix: String::new(),
            media_placeholder: "<image>".to_string(),
        }
    }

    /// Generates a Qwen compliant template.
    pub fn qwen() -> Self {
        Self {
            system_prefix: "<|im_start|>system\n".to_string(),
            system_suffix: "<|im_end|>\n".to_string(),
            user_prefix: "<|im_start|>user\n".to_string(),
            user_suffix: "<|im_end|>\n".to_string(),
            assistant_prefix: "<|im_start|>assistant\n".to_string(),
            assistant_suffix: "<|im_end|>\n".to_string(),
            think_prefix: "<think>\n".to_string(),
            think_suffix: "\n</think>\n".to_string(),
            tool_prefix: "<|im_start|>tool\n".to_string(),
            tool_suffix: "<|im_end|>\n".to_string(),
            tool_id_prefix: String::new(),
            tool_id_suffix: String::new(),
            media_placeholder: "<|media|>".to_string(),
        }
    }

    /// Generates a Mistral compliant template.
    pub fn mistral() -> Self {
        Self {
            system_prefix: "<s>[INST] ".to_string(),
            system_suffix: "[/INST]\n".to_string(),
            user_prefix: "[INST] ".to_string(),
            user_suffix: " [/INST]\n".to_string(),
            assistant_prefix: "".to_string(),
            assistant_suffix: "</s>\n".to_string(),
            think_prefix: "<think>\n".to_string(),
            think_suffix: "\n</think>\n".to_string(),
            tool_prefix: "[AVAILABLE_TOOLS] ".to_string(),
            tool_suffix: "[/AVAILABLE_TOOLS]\n".to_string(),
            tool_id_prefix: String::new(),
            tool_id_suffix: String::new(),
            media_placeholder: "[IMG]".to_string(),
        }
    }

    /// Generates a legacy Llama 2 compliant template.
    pub fn llama2() -> Self {
        Self {
            system_prefix: "<<SYS>>\n".to_string(),
            system_suffix: "\n<</SYS>>\n\n".to_string(),
            user_prefix: "[INST] ".to_string(),
            user_suffix: " [/INST] ".to_string(),
            assistant_prefix: "".to_string(),
            assistant_suffix: " </s><s>".to_string(),
            think_prefix: String::new(),
            think_suffix: String::new(),
            tool_prefix: " [TOOL] ".to_string(),
            tool_suffix: " [/TOOL] ".to_string(),
            tool_id_prefix: String::new(),
            tool_id_suffix: String::new(),
            media_placeholder: String::new(),
        }
    }

    /// Wraps and returns a manually constructed template.
    pub fn custom_template(template: ChatTemplate) -> Self {
        template
    }
}
