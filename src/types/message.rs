// src/types/message.rs

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentPart {
    Text { text: String },
    Image { base64: String },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Message {
    System {
        content: String,
    },
    User {
        content: Vec<ContentPart>,
    },
    Tool {
        content: String,
        tool_id: Option<String>,
    },
    Assistant {
        content: String,

        // (name, args, tool_call_id)
        tool_calls: Vec<(String, serde_json::Value, String)>,
    },
}

impl Message {
    pub fn text_len(&self) -> usize {
        match self {
            Message::System { content } => content.len(),
            Message::User { content } => content
                .iter()
                .map(|p| match p {
                    ContentPart::Text { text } => text.len(),
                    _ => 0,
                })
                .sum(),
            Message::Tool { content, .. } => content.len(),
            Message::Assistant { content, .. } => content.len(),
        }
    }

    pub fn user_multimodal(text: &str, image_base64: &str) -> Self {
        Self::User {
            content: vec![
                ContentPart::Text {
                    text: text.to_string(),
                },
                ContentPart::Image {
                    base64: image_base64.to_string(),
                },
            ],
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::System { content } => write!(f, "{}", content),
            Message::User { content } => {
                let text: String = content
                    .iter()
                    .filter_map(|p| match p {
                        ContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect();
                write!(f, "{}", text)
            }
            Message::Tool { content, .. } => write!(f, "{}", content),
            Message::Assistant { content, .. } => write!(f, "{}", content),
        }
    }
}
