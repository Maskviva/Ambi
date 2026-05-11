// bindings/node/src/message.rs

use ambi::types::{ContentPart, Message};
use napi_derive::napi;

#[napi(object)]
pub struct JsToolCall {
    pub name: String,
    pub arguments: String,
    pub id: String,
}

#[napi(object)]
pub struct JsContentPart {
    pub r#type: String,
    pub text: Option<String>,
    pub base64: Option<String>,
}

#[napi(object)]
pub struct JsMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<JsToolCall>>,
    pub tool_id: Option<String>,
    pub parts: Option<Vec<JsContentPart>>,
}

pub fn convert_content_part(part: &JsContentPart) -> Option<ContentPart> {
    match part.r#type.as_str() {
        "text" => part
            .text
            .as_ref()
            .map(|t: &String| ContentPart::Text { text: t.clone() }),
        "image" => part
            .base64
            .as_ref()
            .map(|b: &String| ContentPart::Image { base64: b.clone() }),
        _ => None,
    }
}

pub fn convert_message(msg: &Message) -> JsMessage {
    match msg {
        Message::System { content } => JsMessage {
            role: "system".to_string(),
            content: content.clone(),
            tool_calls: None,
            tool_id: None,
            parts: None,
        },
        Message::User { content } => {
            let text: String = content
                .iter()
                .filter_map(|p| match p {
                    ContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect();
            let parts: Vec<JsContentPart> = content
                .iter()
                .map(|p| match p {
                    ContentPart::Text { text } => JsContentPart {
                        r#type: "text".to_string(),
                        text: Some(text.clone()),
                        base64: None,
                    },
                    ContentPart::Image { base64 } => JsContentPart {
                        r#type: "image".to_string(),
                        text: None,
                        base64: Some(base64.clone()),
                    },
                })
                .collect();
            JsMessage {
                role: "user".to_string(),
                content: text,
                tool_calls: None,
                tool_id: None,
                parts: Some(parts),
            }
        }
        Message::Tool { content, tool_id } => JsMessage {
            role: "tool".to_string(),
            content: content.clone(),
            tool_calls: None,
            tool_id: tool_id.clone(),
            parts: None,
        },
        Message::Assistant {
            content,
            tool_calls,
        } => {
            let calls: Vec<JsToolCall> = tool_calls
                .iter()
                .map(|(name, args, id)| JsToolCall {
                    name: name.clone(),
                    arguments: args.to_string(),
                    id: id.clone(),
                })
                .collect();
            JsMessage {
                role: "assistant".to_string(),
                content: content.clone(),
                tool_calls: if calls.is_empty() { None } else { Some(calls) },
                tool_id: None,
                parts: None,
            }
        }
    }
}
