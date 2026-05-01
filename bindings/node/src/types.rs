// bindings/node/src/types.rs
//
//! Every data type the Ambi framework moves around — messages, content
//! segments, tool definitions, request payloads.
//!
//! JS users construct these the same way they construct Rust structs:
//! ```js
//! const msg = { role: "user", content: "hello" };
//! const part = { partType: "text", text: "Hi" };
//! ```
//! Each type carries a bidirectional conversion to its Rust counterpart,
//! so the engine can work with them natively.

use napi_derive::napi;
use serde_json::Value as JsonValue;

// ── ContentPart ──

/// A single segment inside a message body — either plain text or an
/// image payload. Mirrors the Rust `ContentPart` enum faithfully.
#[napi(object)]
pub struct JsContentPart {
    /// `"text"` or `"image"`
    pub part_type: String,
    /// Text body (set when `part_type == "text"`).
    pub text: Option<String>,
    /// Base64-encoded image, or a URL pointing to one (set when
    /// `part_type == "image"`).
    pub base64: Option<String>,
}

impl From<JsContentPart> for ambi::ContentPart {
    fn from(part: JsContentPart) -> Self {
        match part.part_type.as_str() {
            "image" => ambi::ContentPart::Image {
                base64: part.base64.unwrap_or_default(),
            },
            _ => ambi::ContentPart::Text {
                text: part.text.unwrap_or_default(),
            },
        }
    }
}

impl From<&ambi::ContentPart> for JsContentPart {
    fn from(part: &ambi::ContentPart) -> Self {
        match part {
            ambi::ContentPart::Text { text } => Self {
                part_type: "text".to_string(),
                text: Some(text.clone()),
                base64: None,
            },
            ambi::ContentPart::Image { base64 } => Self {
                part_type: "image".to_string(),
                text: None,
                base64: Some(base64.clone()),
            },
        }
    }
}

// ── ToolCall ──

/// A tool invocation requested by the model — name, JSON arguments,
/// and a unique call ID for correlating results.
#[napi(object)]
pub struct JsToolCall {
    pub name: String,
    pub arguments: JsonValue,
    pub id: String,
}

// ── Message ──

/// One turn in the conversation history.
///
/// A plain JS object with a `role` discriminator:
/// - `"system"`       → system instruction (`content`)
/// - `"user"`         → user input (`content` or `contentParts` for images)
/// - `"assistant"`    → model reply (`content`, optionally `toolCalls`)
/// - `"tool"`         → tool execution result (`content`, `toolId`)
#[napi(object)]
pub struct JsMessage {
    /// One of `"system" | "user" | "assistant" | "tool"`.
    pub role: String,
    /// Text payload — the most common field across all roles.
    pub content: Option<String>,
    /// Multimodal content parts. When set, the user message carries
    /// images alongside text.
    pub content_parts: Option<Vec<JsContentPart>>,
    /// Parsed tool calls from the assistant.
    pub tool_calls: Option<Vec<JsToolCall>>,
    /// Links a tool message back to the assistant's call that triggered it.
    pub tool_id: Option<String>,
}

impl JsMessage {
    /// Build a system instruction.
    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: Some(content.to_string()),
            content_parts: None,
            tool_calls: None,
            tool_id: None,
        }
    }

    /// Build a plain-text user message.
    pub fn user(text: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(text.to_string()),
            content_parts: None,
            tool_calls: None,
            tool_id: None,
        }
    }

    /// Build a multimodal user message that carries an image alongside text.
    pub fn user_multimodal(text: &str, image_base64: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: None,
            content_parts: Some(vec![
                JsContentPart {
                    part_type: "text".to_string(),
                    text: Some(text.to_string()),
                    base64: None,
                },
                JsContentPart {
                    part_type: "image".to_string(),
                    text: None,
                    base64: Some(image_base64.to_string()),
                },
            ]),
            tool_calls: None,
            tool_id: None,
        }
    }

    /// Build an assistant message, optionally bundling tool calls.
    pub fn assistant(content: &str, tool_calls: Option<Vec<JsToolCall>>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Some(content.to_string()),
            content_parts: None,
            tool_calls,
            tool_id: None,
        }
    }

    /// Build a tool result message.
    pub fn tool_result(content: &str, tool_id: Option<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content.to_string()),
            content_parts: None,
            tool_calls: None,
            tool_id,
        }
    }
}

// ── Conversion: JsMessage ↔ Rust Message ──

impl TryFrom<JsMessage> for ambi::Message {
    type Error = String;

    fn try_from(js: JsMessage) -> Result<Self, Self::Error> {
        match js.role.as_str() {
            "system" => Ok(ambi::Message::System {
                content: js.content.unwrap_or_default(),
            }),
            "user" => {
                if let Some(parts) = js.content_parts {
                    let content: Vec<ambi::ContentPart> =
                        parts.into_iter().map(|p| p.into()).collect();
                    Ok(ambi::Message::User { content })
                } else {
                    Ok(ambi::Message::User {
                        content: vec![ambi::ContentPart::Text {
                            text: js.content.unwrap_or_default(),
                        }],
                    })
                }
            }
            "assistant" => {
                let tool_calls: Vec<(String, JsonValue, String)> = js
                    .tool_calls
                    .unwrap_or_default()
                    .into_iter()
                    .map(|tc| (tc.name, tc.arguments, tc.id))
                    .collect();
                Ok(ambi::Message::Assistant {
                    content: js.content.unwrap_or_default(),
                    tool_calls,
                })
            }
            "tool" => Ok(ambi::Message::Tool {
                content: js.content.unwrap_or_default(),
                tool_id: js.tool_id,
            }),
            other => Err(format!("Unknown message role: {}", other)),
        }
    }
}

impl From<&ambi::Message> for JsMessage {
    fn from(msg: &ambi::Message) -> Self {
        match msg {
            ambi::Message::System { content } => Self {
                role: "system".to_string(),
                content: Some(content.clone()),
                content_parts: None,
                tool_calls: None,
                tool_id: None,
            },
            ambi::Message::User { content } => {
                let parts: Vec<JsContentPart> = content.iter().map(|p| p.into()).collect();
                Self {
                    role: "user".to_string(),
                    content: None,
                    content_parts: Some(parts),
                    tool_calls: None,
                    tool_id: None,
                }
            }
            ambi::Message::Assistant {
                content,
                tool_calls,
            } => {
                let calls: Vec<JsToolCall> = tool_calls
                    .iter()
                    .map(|(name, args, id)| JsToolCall {
                        name: name.clone(),
                        arguments: args.clone(),
                        id: id.clone(),
                    })
                    .collect();
                Self {
                    role: "assistant".to_string(),
                    content: Some(content.clone()),
                    content_parts: None,
                    tool_calls: Some(calls),
                    tool_id: None,
                }
            }
            ambi::Message::Tool { content, tool_id } => Self {
                role: "tool".to_string(),
                content: Some(content.clone()),
                content_parts: None,
                tool_calls: None,
                tool_id: tool_id.clone(),
            },
        }
    }
}

// ── ToolDefinition ──

/// Describes a tool to the LLM — what it does, what arguments it expects,
/// and how to handle timeouts.
///
/// Mirrors `ambi::types::ToolDefinition` exactly so no semantic gap
/// exists between JS and Rust tool registration.
#[napi(object)]
pub struct JsToolDefinition {
    pub name: String,
    pub description: String,
    /// JSON Schema — the LLM uses this to figure out what to pass.
    pub parameters: JsonValue,
    /// Execution timeout in seconds. Defaults to 15 if omitted.
    pub timeout_secs: Option<u32>,
    /// How many retries on timeout (only respected for idempotent tools).
    pub max_retries: Option<u32>,
    /// Whether the tool is safe to re-run on failure.
    pub is_idempotent: bool,
}

impl From<JsToolDefinition> for ambi::types::ToolDefinition {
    fn from(def: JsToolDefinition) -> Self {
        ambi::types::ToolDefinition {
            name: def.name,
            description: def.description,
            parameters: def.parameters,
            timeout_secs: def.timeout_secs.map(|s| s as u64),
            max_retries: def.max_retries.map(|r| r as usize),
            is_idempotent: def.is_idempotent,
        }
    }
}

impl From<&ambi::types::ToolDefinition> for JsToolDefinition {
    fn from(def: &ambi::types::ToolDefinition) -> Self {
        Self {
            name: def.name.clone(),
            description: def.description.clone(),
            parameters: def.parameters.clone(),
            timeout_secs: def.timeout_secs.map(|s| s as u32),
            max_retries: def.max_retries.map(|r| r as u32),
            is_idempotent: def.is_idempotent,
        }
    }
}

// ── LLMRequest ──

/// The full payload the engine sends to the LLM.
///
/// Exposing this directly means JS users can call `engine.chat(request)`
/// without going through the Agent pipeline — handy for quick experiments
/// or custom orchestration patterns.
#[napi(object)]
pub struct JsLlmRequest {
    /// The assembled system preamble.
    pub system_prompt: String,
    /// Filtered conversation history.
    pub history: Vec<JsMessage>,
    /// Tool definitions available to the model.
    pub tools: Vec<JsToolDefinition>,
    /// Raw tool-calling instruction injected into the prompt.
    pub tool_prompt: String,
    /// The complete, rendered prompt string (used by local engines).
    pub formatted_prompt: String,
    /// Tool-call delimiters, as a two-element array `[start_tag, end_tag]`.
    pub tool_tags: Vec<String>,
    /// Base64-encoded images extracted from the history.
    pub images: Vec<String>,
}

impl TryFrom<JsLlmRequest> for ambi::types::LLMRequest {
    type Error = String;

    fn try_from(js: JsLlmRequest) -> Result<Self, Self::Error> {
        let mut history = Vec::new();
        for js_msg in js.history {
            history.push(std::sync::Arc::new(js_msg.try_into()?));
        }

        let tool_tags = if js.tool_tags.len() >= 2 {
            (js.tool_tags[0].clone(), js.tool_tags[1].clone())
        } else {
            (String::new(), String::new())
        };

        Ok(ambi::types::LLMRequest {
            system_prompt: js.system_prompt,
            history,
            tools: js.tools.into_iter().map(|t| t.into()).collect(),
            tool_prompt: js.tool_prompt,
            formatted_prompt: js.formatted_prompt,
            tool_tags,
            images: js.images,
        })
    }
}
