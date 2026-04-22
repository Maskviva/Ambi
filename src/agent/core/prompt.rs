use super::{Agent, CompletionRequest};
use crate::llm::{ChatTemplate, LLMRequest};
use crate::types::message::Message;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

impl Agent {
    pub(crate) async fn get_llm_request(
        req_mutex: &TokioMutex<CompletionRequest>,
        system_prompt: &str,
        tpl: &ChatTemplate,
        cached_tool_prompt: &str,
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

        let formatted_prompt = Self::build_prompt(
            &final_system_prompt,
            &filtered_history,
            tpl,
            cached_tool_prompt,
        );

        LLMRequest {
            system_prompt: final_system_prompt,
            history: filtered_history,
            tool_prompt: cached_tool_prompt.to_string(),
            formatted_prompt,
        }
    }

    pub(super) fn build_prompt(
        system_prompt: &str,
        filtered_history: &[Arc<Message>],
        tpl: &ChatTemplate,
        tool_content: &str,
    ) -> String {
        let mut prompt = String::with_capacity(2048);

        if !system_prompt.is_empty() || !tool_content.is_empty() {
            prompt.push_str(&tpl.system_prefix);
            prompt.push_str(system_prompt);

            if !system_prompt.is_empty() && !tool_content.is_empty() {
                prompt.push_str("\n\n");
            }

            prompt.push_str(tool_content);
            prompt.push_str(&tpl.system_suffix);
        }

        for msg in filtered_history {
            match &**msg {
                Message::User { .. } => {
                    prompt.push_str(&tpl.user_prefix);
                    msg.write_text_to(&mut prompt);
                    prompt.push_str(&tpl.user_suffix);
                }
                Message::Tool { content } => {
                    prompt.push_str(&tpl.tool_prefix);
                    prompt.push_str(content);
                    prompt.push_str(&tpl.tool_suffix);
                }
                Message::Assistant { content } => {
                    prompt.push_str(&tpl.assistant_prefix);
                    prompt.push_str(content);
                    prompt.push_str(&tpl.assistant_suffix);
                }
                _ => {}
            }
        }
        prompt.push_str(&tpl.assistant_prefix);
        prompt
    }
}
