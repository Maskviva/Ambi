// src/llm/providers/openai/translator.rs

use super::OpenAIEngine;
use crate::error::{AmbiError, Result};
use crate::types::message::Message;
use crate::types::LLMRequest;
use crate::ContentPart;
use async_openai::types::chat::{
    ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImageArgs,
    ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs, ChatCompletionTool,
    ChatCompletionTools, CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    FunctionObjectArgs, ImageUrlArgs,
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
            let api_msg: ChatCompletionRequestMessage = match &**msg {
                Message::User { content } => {
                    let mut parts = Vec::new();
                    for part in content {
                        match part {
                            ContentPart::Text { text } => {
                                parts.push(
                                    ChatCompletionRequestMessageContentPartTextArgs::default()
                                        .text(text.clone())
                                        .build()
                                        .map_err(|e| AmbiError::EngineError(e.to_string()))?
                                        .into(),
                                );
                            }
                            ContentPart::Image { url } => {
                                parts.push(
                                    ChatCompletionRequestMessageContentPartImageArgs::default()
                                        .image_url(
                                            ImageUrlArgs::default()
                                                .url(url.clone())
                                                .build()
                                                .map_err(|e| {
                                                    AmbiError::EngineError(e.to_string())
                                                })?,
                                        )
                                        .build()
                                        .map_err(|e| AmbiError::EngineError(e.to_string()))?
                                        .into(),
                                );
                            }
                        }
                    }

                    ChatCompletionRequestUserMessageArgs::default()
                        .content(parts)
                        .build()
                        .map_err(|e| AmbiError::EngineError(e.to_string()))?
                        .into()
                }
                Message::Assistant {
                    content,
                    tool_calls,
                } => {
                    let mut args = ChatCompletionRequestAssistantMessageArgs::default();
                    if !content.is_empty() {
                        args.content(content.clone());
                    }
                    if !tool_calls.is_empty() {
                        let api_tool_calls: Vec<ChatCompletionMessageToolCalls> = tool_calls
                            .iter()
                            .map(|(name, arg, id)| {
                                serde_json::from_value(serde_json::json!({
                                    "id": id,
                                    "type": "function",
                                    "function": {
                                        "name": name,
                                        "arguments": arg.to_string()
                                    }
                                }))
                                .expect("Failed to deserialize tool call safely")
                            })
                            .collect();
                        args.tool_calls(api_tool_calls);
                    }
                    args.build()
                        .map_err(|e| AmbiError::EngineError(e.to_string()))?
                        .into()
                }
                Message::Tool { content, tool_id } => {
                    let id = tool_id
                        .clone()
                        .unwrap_or_else(|| "call_default".to_string());
                    ChatCompletionRequestToolMessageArgs::default()
                        .tool_call_id(id)
                        .content(content.clone())
                        .build()
                        .map_err(|e| AmbiError::EngineError(e.to_string()))?
                        .into()
                }
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
