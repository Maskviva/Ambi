use super::{Agent, CompletionRequest};
use crate::agent::tool::{ToolDefinition, ToolManager};
use crate::llm::{ChatTemplate, LLMRequest};
use crate::types::message::Message;
use std::fmt::Write;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

impl Agent {
    pub(crate) async fn get_llm_request(
        req_mutex: &TokioMutex<CompletionRequest>,
        system_prompt: &str,
        tpl: &ChatTemplate,
        tools: &[ToolDefinition],
    ) -> LLMRequest {
        let req = req_mutex.lock().await;

        let mut final_system_prompt = system_prompt.to_string();
        let mut filtered_history = Vec::new();

        for msg in req.chat_history.all() {
            match &**msg {
                Message::System { content } => {
                    if !final_system_prompt.is_empty() {
                        final_system_prompt.push_str("\n\n");
                    }
                    final_system_prompt.push_str(content);
                }
                _ => {
                    filtered_history.push(Arc::clone(msg));
                }
            }
        }

        let formatted_prompt =
            Self::build_prompt(&final_system_prompt, &filtered_history, tpl, tools);
        let tool_prompt = ToolManager::tool_prompt(tools.to_vec());

        LLMRequest {
            system_prompt: final_system_prompt,
            history: filtered_history,
            tool_prompt,
            formatted_prompt,
        }
    }

    pub(super) fn build_prompt(
        system_prompt: &str,
        filtered_history: &[Arc<Message>],
        tpl: &ChatTemplate,
        tools: &[ToolDefinition],
    ) -> String {
        let mut prompt = String::with_capacity(2048);

        let mut combined_system = system_prompt.to_string();
        let tool_content = ToolManager::tool_prompt(tools.to_vec());

        if !tool_content.is_empty() {
            if !combined_system.is_empty() {
                combined_system.push_str("\n\n");
            }
            combined_system.push_str(&tool_content);
        }

        if !combined_system.is_empty() {
            let _ = write!(
                prompt,
                "{}{}{}",
                tpl.system_prefix, combined_system, tpl.system_suffix
            );
        }

        for msg in filtered_history {
            match &**msg {
                Message::User { .. } => {
                    let text = msg.get_text_content();
                    let _ = write!(prompt, "{}{}{}", tpl.user_prefix, text, tpl.user_suffix);
                }
                Message::Tool { content } => {
                    let _ = write!(prompt, "{}{}{}", tpl.tool_prefix, content, tpl.tool_suffix);
                }
                Message::Assistant { content } => {
                    let _ = write!(
                        prompt,
                        "{}{}{}",
                        tpl.assistant_prefix, content, tpl.assistant_suffix
                    );
                }
                _ => {}
            }
        }
        prompt.push_str(&tpl.assistant_prefix);
        prompt
    }
}
