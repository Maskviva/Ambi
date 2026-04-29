// src/agent/core/prompt.rs

use super::{Agent, AgentState};
use crate::types::{ChatTemplate, LLMRequest, Message, ToolDefinition};
use crate::ContentPart;
use std::sync::Arc;

impl Agent {
    pub(crate) fn get_llm_request(
        state: &AgentState,
        system_prompt: &str,
        tpl: &ChatTemplate,
        tools: &[ToolDefinition],
        cached_tool_prompt: &str,
        tool_tags: (String, String),
    ) -> LLMRequest {
        let mut system_prompts_buffer = Vec::new();
        let mut filtered_history = Vec::new();
        let mut extracted_images = Vec::new();

        if !system_prompt.is_empty() {
            system_prompts_buffer.push(system_prompt.to_string());
        }

        for (msg, _) in state.chat_history.all() {
            match &**msg {
                Message::System { content } => {
                    system_prompts_buffer.push(content.clone());
                }
                Message::User { content } => {
                    for part in content {
                        if let ContentPart::Image { base64 } = part {
                            extracted_images.push(base64.clone());
                        }
                    }
                    filtered_history.push(Arc::clone(msg));
                }
                _ => {
                    filtered_history.push(Arc::clone(msg));
                }
            }
        }

        let final_system_prompt = system_prompts_buffer.join("\n\n");

        let formatted_prompt = Self::build_prompt(
            &final_system_prompt,
            &filtered_history,
            tpl,
            cached_tool_prompt,
        );

        LLMRequest {
            system_prompt: final_system_prompt,
            history: filtered_history,
            tools: tools.to_vec(),
            tool_prompt: cached_tool_prompt.to_string(),
            formatted_prompt,
            tool_tags,
            images: extracted_images,
        }
    }

    pub(super) fn build_prompt(
        system_prompt: &str,
        filtered_history: &[Arc<Message>],
        tpl: &ChatTemplate,
        tool_content: &str,
    ) -> String {
        let mut prompt = String::with_capacity(2048);

        // --- Render Preamble (System + Tools) ---
        if !system_prompt.is_empty() || !tool_content.is_empty() {
            prompt.push_str(&tpl.system_prefix);
            prompt.push_str(system_prompt);

            if !system_prompt.is_empty() && !tool_content.is_empty() {
                prompt.push_str("\n\n");
            }

            prompt.push_str(tool_content);
            prompt.push_str(&tpl.system_suffix);
        }

        // --- Render Conversation History ---
        for msg in filtered_history {
            match &**msg {
                Message::User { content } => {
                    prompt.push_str(&tpl.user_prefix);
                    let mut user_text = String::new();
                    let mut has_image = false;

                    for part in content {
                        match part {
                            ContentPart::Text { text } => user_text.push_str(text),
                            ContentPart::Image { .. } => has_image = true,
                        }
                    }
                    prompt.push_str(&user_text);

                    if has_image && !tpl.media_placeholder.is_empty() {
                        prompt.push_str(&tpl.media_placeholder);
                    }
                    prompt.push_str(&tpl.user_suffix);
                }
                Message::Tool { content, tool_id } => {
                    prompt.push_str(&tpl.tool_prefix);
                    if let Some(id) = tool_id {
                        if !tpl.tool_id_prefix.is_empty() || !tpl.tool_id_suffix.is_empty() {
                            prompt.push_str(&tpl.tool_id_prefix);
                            prompt.push_str(id);
                            prompt.push_str(&tpl.tool_id_suffix);
                        }
                    }
                    prompt.push_str(content);
                    prompt.push_str(&tpl.tool_suffix);
                }
                Message::Assistant { content, .. } => {
                    prompt.push_str(&tpl.assistant_prefix);
                    prompt.push_str(content);
                    prompt.push_str(&tpl.assistant_suffix);
                }
                _ => {} // System Message already handled above
            }
        }

        // --- Render Assistant Generation Prompt ---
        prompt.push_str(&tpl.assistant_prefix);
        prompt
    }
}
