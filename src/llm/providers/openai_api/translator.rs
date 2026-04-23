// src/llm/providers/openai/translator.rs
use super::OpenAIEngine;
use crate::error::{AmbiError, Result};
use crate::types::message::Message;
use crate::types::LLMRequest;
use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionTool, ChatCompletionTools, CreateChatCompletionRequest,
    CreateChatCompletionRequestArgs, FunctionObjectArgs,
};

impl OpenAIEngine {
    pub(super) fn get_request(
        &self,
        model_name: String,
        request: LLMRequest,
        stream: bool,
    ) -> Result<CreateChatCompletionRequest> {
        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        if !request.system_prompt.is_empty() {
            messages.push(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(request.system_prompt.clone())
                    .build()
                    .map_err(|e| AmbiError::EngineError(e.to_string()))?
                    .into(),
            );
        }

        for msg in &request.history {
            let text = msg.to_string();
            let api_msg: ChatCompletionRequestMessage = match &**msg {
                Message::User { .. } => ChatCompletionRequestUserMessageArgs::default()
                    .content(text)
                    .build()
                    .map_err(|e| AmbiError::EngineError(e.to_string()))?
                    .into(),
                Message::Assistant { .. } => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(text)
                    .build()
                    .map_err(|e| AmbiError::EngineError(e.to_string()))?
                    .into(),
                Message::Tool { .. } => ChatCompletionRequestUserMessageArgs::default()
                    .content(format!("Tool result: {}", text))
                    .build()
                    .map_err(|e| AmbiError::EngineError(e.to_string()))?
                    .into(),
                Message::System { .. } => continue,
            };
            messages.push(api_msg);
        }

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(model_name)
            .messages(messages)
            .temperature(self.cfg.temp)
            .top_p(self.cfg.top_p)
            .stream(stream);

        if !request.tools.is_empty() {
            let mut api_tools = Vec::new();
            for t in &request.tools {
                let func = FunctionObjectArgs::default()
                    .name(&t.name)
                    .description(&t.description)
                    .parameters(t.parameters.clone())
                    .build()
                    .map_err(|e| AmbiError::EngineError(e.to_string()))?;

                let tool = ChatCompletionTool { function: func };
                api_tools.push(ChatCompletionTools::Function(tool));
            }
            request_builder.tools(api_tools);
        }

        request_builder.build().map_err(|e| {
            AmbiError::EngineError(format!("Failed to build OpenAI API request: {}", e))
        })
    }
}
