use crate::agent::core::formatter::TagStreamFormatter;
use crate::agent::tool::{StreamFormatter, ToolCallParser};
use serde_json::Value;

pub struct TagToolParser {
    pub start_tag: String,
    pub end_tag: String,
}

impl TagToolParser {
    pub fn new(start_tag: &str, end_tag: &str) -> Self {
        Self {
            start_tag: start_tag.to_string(),
            end_tag: end_tag.to_string(),
        }
    }

    fn clean_markdown_json(raw: &str) -> &str {
        let mut s = raw.trim();
        if s.starts_with("```json") {
            s = &s[7..];
        } else if s.starts_with("```") {
            s = &s[3..];
        }
        s = s.trim();
        if s.ends_with("```") {
            s = &s[..s.len() - 3];
        }
        s.trim()
    }

    fn parser_str(calls: &mut Vec<(String, Value)>, str: &str) -> bool {
        if let Ok(val) = serde_json::from_str::<Value>(str) {
            let mut process_item = |item: &Value| {
                if let (Some(name), Some(args)) =
                    (item.get("name").and_then(|n| n.as_str()), item.get("args"))
                {
                    calls.push((name.to_string(), args.clone()));
                }
            };

            if val.is_object() {
                process_item(&val);
            } else if let Some(arr) = val.as_array() {
                for item in arr {
                    process_item(item);
                }
            }
            true
        } else {
            false
        }
    }

    fn extract_and_push_call(json_str: &str, calls: &mut Vec<(String, Value)>) {
        let trimmed = json_str.trim();
        if trimmed.is_empty() {
            return;
        }

        if Self::parser_str(calls, trimmed) {
            return;
        }

        if let Some(last_brace) = trimmed.rfind('}') {
            let truncated = &trimmed[..=last_brace];
            if Self::parser_str(calls, truncated) {
                return;
            }
        }

        log::warn!("Failed to parse Tool JSON: {}", trimmed);
        calls.push((
            "__format_error__".to_string(),
            serde_json::json!({
                "error": "Invalid JSON syntax",
                "raw": trimmed
            }),
        ));
    }
}

impl ToolCallParser for TagToolParser {
    fn get_tags(&self) -> (String, String) {
        (self.start_tag.clone(), self.end_tag.clone())
    }
    fn format_instruction(&self, tools_json: &str) -> String {
        format!(
            "You can use tools. Call format:\n{}{{\"name\":\"tool_name\",\"args\":{{...}}}}{}\nAvailable tools:\n{}",
            self.start_tag, self.end_tag, tools_json
        )
    }

    fn parse(&self, text: &str) -> Vec<(String, Value)> {
        let mut calls = Vec::new();
        let mut current_text = text;

        while let Some(start) = current_text.find(&self.start_tag) {
            let content_start = start + self.start_tag.len();

            if let Some(end_offset) = current_text[content_start..].find(&self.end_tag) {
                let end = content_start + end_offset;
                let clean_json = Self::clean_markdown_json(&current_text[content_start..end]);
                Self::extract_and_push_call(clean_json, &mut calls);
                current_text = &current_text[end + self.end_tag.len()..];
            } else {
                let clean_json = Self::clean_markdown_json(&current_text[content_start..]);
                Self::extract_and_push_call(clean_json, &mut calls);
                break;
            }
        }
        calls
    }

    fn create_stream_formatter(&self) -> Box<dyn StreamFormatter> {
        Box::new(TagStreamFormatter::new(&self.start_tag, &self.end_tag))
    }
}

pub struct DefaultToolParser;
impl DefaultToolParser {
    pub fn make() -> TagToolParser {
        TagToolParser::new("[TOOL_CALL]", "[/TOOL_CALL]")
    }
}
