// src/llm/providers/openai_api/translator.rs

//! Request/response translators between Ambi's internal types and the OpenAI API schema.
//!
//! Converts `LLMRequest` into `CreateChatCompletionRequest`, handling system prompts,
//! multi-modal content (text + image), tool definitions, and assistant tool calls.

use super::OpenAIEngine;
use crate::error::{AmbiError, Result};
use crate::types::{LLMRequest, Message};
use crate::ContentPart;
use async_openai::types::chat::{
    ChatCompletionMessageToolCall, ChatCompletionMessageToolCalls,
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPartImageArgs,
    ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs, ChatCompletionTool,
    ChatCompletionTools, CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    FunctionCall, FunctionObjectArgs, ImageUrlArgs,
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
                    .map_err(|e| AmbiError::engine(e.to_string()))?
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
                                        .map_err(|e| AmbiError::engine(e.to_string()))?
                                        .into(),
                                );
                            }
                            ContentPart::Image { base64 } => {
                                let safe_url = if base64.starts_with("data:image")
                                    || base64.starts_with("http")
                                {
                                    base64.clone()
                                } else {
                                    format!("data:image/jpeg;base64,{}", base64)
                                };

                                parts.push(
                                    ChatCompletionRequestMessageContentPartImageArgs::default()
                                        .image_url(
                                            ImageUrlArgs::default()
                                                .url(safe_url)
                                                .build()
                                                .map_err(|e| AmbiError::engine(e.to_string()))?,
                                        )
                                        .build()
                                        .map_err(|e| AmbiError::engine(e.to_string()))?
                                        .into(),
                                );
                            }
                        }
                    }

                    ChatCompletionRequestUserMessageArgs::default()
                        .content(parts)
                        .build()
                        .map_err(|e| AmbiError::engine(e.to_string()))?
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
                        // OpenAI requires `function.arguments` to be a JSON *string*, while Ambi
                        // stores the arguments as an already-parsed `Value`. Serialize exactly
                        // once here so the value round-trips instead of being double-encoded.
                        // Built as a typed value rather than re-parsed from JSON, so a malformed
                        // argument can never panic on the request path.
                        let api_tool_calls: Vec<ChatCompletionMessageToolCalls> = tool_calls
                            .iter()
                            .map(|(name, arg, id)| {
                                ChatCompletionMessageToolCall {
                                    id: id.clone(),
                                    function: FunctionCall {
                                        name: name.clone(),
                                        arguments: arg.to_string(),
                                    },
                                }
                                .into()
                            })
                            .collect();
                        args.tool_calls(api_tool_calls);
                    }
                    args.build()
                        .map_err(|e| AmbiError::engine(e.to_string()))?
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
                        .map_err(|e| AmbiError::engine(e.to_string()))?
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
                    .map_err(|e| AmbiError::engine(e.to_string()))?;

                let tool = ChatCompletionTool { function: func };
                api_tools.push(ChatCompletionTools::Function(tool));
            }
            request_builder.tools(api_tools);
        }

        request_builder
            .build()
            .map_err(|e| AmbiError::engine(format!("Failed to build OpenAI API request: {}", e)))
    }
}

/// Renders one native tool call back into Ambi's tag format,
/// `[start]{"name":"...","args":{...}}[end]`.
///
/// `arguments` is a JSON string on the wire. It is parsed and re-serialized as a JSON value so the
/// tag carries real JSON (`"args":{"k":"v"}`) rather than a quoted string. Arguments the model did
/// not produce as valid JSON are passed through verbatim, which the tool parser already reports as
/// a format error instead of silently dropping the call. Custom tool calls have no function schema
/// Ambi can execute, so they are skipped.
pub(super) fn render_tool_call(
    call: &ChatCompletionMessageToolCalls,
    start_tag: &str,
    end_tag: &str,
) -> Option<String> {
    let ChatCompletionMessageToolCalls::Function(call) = call else {
        return None;
    };

    let args = serde_json::from_str::<serde_json::Value>(&call.function.arguments)
        .unwrap_or(serde_json::Value::String(call.function.arguments.clone()));

    Some(format!(
        "{start_tag}{body}{end_tag}",
        body = serde_json::json!({ "name": call.function.name, "args": args })
    ))
}
