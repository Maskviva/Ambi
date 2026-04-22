use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentPart {
    Text { text: String },
    Image { url: String },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Message {
    System { content: String },
    User { content: Vec<ContentPart> },
    Tool { content: String },
    Assistant { content: String },
}

impl Message {
    pub fn user_text(text: &str) -> Self {
        Self::User {
            content: vec![ContentPart::Text {
                text: text.to_string(),
            }],
        }
    }

    pub fn get_text_content(&self) -> String {
        match self {
            Message::System { content } => content.clone(),
            Message::User { content } => {
                let mut text = String::new();
                for part in content {
                    if let ContentPart::Text { text: t } = part {
                        text.push_str(t);
                    }
                }
                text
            }
            Message::Tool { content } => content.clone(),
            Message::Assistant { content } => content.clone(),
        }
    }
}
