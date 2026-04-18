use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChatTemplateType {
    #[default]
    Chatml,
    Llama3,
    Gemma,
    Phi3,
    Zephyr,
    Deepseek,
    Mistral,
    Llama2,
}

impl ChatTemplateType {
    pub fn as_template(&self) -> ChatTemplate {
        match self {
            ChatTemplateType::Chatml => ChatTemplate::chatml(),
            ChatTemplateType::Llama3 => ChatTemplate::llama3(),
            ChatTemplateType::Gemma => ChatTemplate::gemma(),
            ChatTemplateType::Phi3 => ChatTemplate::phi3(),
            ChatTemplateType::Zephyr => ChatTemplate::zephyr(),
            ChatTemplateType::Deepseek => ChatTemplate::deepseek(),
            ChatTemplateType::Mistral => ChatTemplate::mistral(),
            ChatTemplateType::Llama2 => ChatTemplate::llama2(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ChatTemplate {
    pub system_prefix: String,
    pub system_suffix: String,
    pub user_prefix: String,
    pub user_suffix: String,
    pub assistant_prefix: String,
    pub assistant_suffix: String,
    pub tool_prefix: String,
    pub tool_suffix: String,
}

impl ChatTemplate {
    pub fn chatml() -> Self {
        Self {
            system_prefix: "<|im_start|>system\n".to_string(),
            system_suffix: "<|im_end|>\n".to_string(),
            user_prefix: "<|im_start|>user\n".to_string(),
            user_suffix: "<|im_end|>\n".to_string(),
            assistant_prefix: "<|im_start|>assistant\n".to_string(),
            assistant_suffix: "<|im_end|>\n".to_string(),
            tool_prefix: "<|im_start|>tool\n".to_string(),
            tool_suffix: "<|im_end|>\n".to_string(),
        }
    }

    pub fn llama3() -> Self {
        Self {
            system_prefix: "<|start_header_id|>system<|end_header_id|>\n\n".to_string(),
            system_suffix: "<|eot_id|>\n".to_string(),
            user_prefix: "<|start_header_id|>user<|end_header_id|>\n\n".to_string(),
            user_suffix: "<|eot_id|>\n".to_string(),
            assistant_prefix: "<|start_header_id|>assistant<|end_header_id|>\n\n".to_string(),
            assistant_suffix: "<|eot_id|>\n".to_string(),
            tool_prefix: "<|start_header_id|>tool<|end_header_id|>\n\n".to_string(),
            tool_suffix: "<|eot_id|>\n".to_string(),
        }
    }

    pub fn gemma() -> Self {
        Self {
            system_prefix: "<start_of_turn>system\n".to_string(),
            system_suffix: "<end_of_turn>\n".to_string(),
            user_prefix: "<start_of_turn>user\n".to_string(),
            user_suffix: "<end_of_turn>\n".to_string(),
            assistant_prefix: "<start_of_turn>model\n".to_string(),
            assistant_suffix: "<end_of_turn>\n".to_string(),
            tool_prefix: "<start_of_turn>tool\n".to_string(),
            tool_suffix: "<end_of_turn>\n".to_string(),
        }
    }

    pub fn phi3() -> Self {
        Self {
            system_prefix: "<|system|>\n".to_string(),
            system_suffix: "<|end|>\n".to_string(),
            user_prefix: "<|user|>\n".to_string(),
            user_suffix: "<|end|>\n".to_string(),
            assistant_prefix: "<|assistant|>\n".to_string(),
            assistant_suffix: "<|end|>\n".to_string(),
            tool_prefix: "<|tool|>\n".to_string(),
            tool_suffix: "<|end|>\n".to_string(),
        }
    }

    pub fn zephyr() -> Self {
        Self {
            system_prefix: "<|system|>\n".to_string(),
            system_suffix: "</s>\n".to_string(),
            user_prefix: "<|user|>\n".to_string(),
            user_suffix: "</s>\n".to_string(),
            assistant_prefix: "<|assistant|>\n".to_string(),
            assistant_suffix: "</s>\n".to_string(),
            tool_prefix: "<|tool|>\n".to_string(),
            tool_suffix: "</s>\n".to_string(),
        }
    }

    pub fn deepseek() -> Self {
        Self {
            system_prefix: "".to_string(),
            system_suffix: "\n\n".to_string(),
            user_prefix: "User: ".to_string(),
            user_suffix: "\n\n".to_string(),
            assistant_prefix: "Assistant: ".to_string(),
            assistant_suffix: "<｜end of sentence｜>".to_string(),
            tool_prefix: "Tool: ".to_string(),
            tool_suffix: "\n\n".to_string(),
        }
    }

    pub fn mistral() -> Self {
        Self {
            system_prefix: "[INST] ".to_string(),
            system_suffix: "\n".to_string(),
            user_prefix: "".to_string(),
            user_suffix: " [/INST]".to_string(),
            assistant_prefix: " ".to_string(),
            assistant_suffix: "</s> ".to_string(),
            tool_prefix: "[TOOL_RESULTS] ".to_string(),
            tool_suffix: " [/TOOL_RESULTS]".to_string(),
        }
    }

    pub fn llama2() -> Self {
        Self {
            system_prefix: "<<SYS>>\n".to_string(),
            system_suffix: "\n<</SYS>>\n\n".to_string(),
            user_prefix: "[INST] ".to_string(),
            user_suffix: " [/INST] ".to_string(),
            assistant_prefix: "".to_string(),
            assistant_suffix: " </s><s>".to_string(),
            tool_prefix: " [TOOL] ".to_string(),
            tool_suffix: " [/TOOL] ".to_string(),
        }
    }
}
