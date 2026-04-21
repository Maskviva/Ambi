use crate::agent::tool::ToolCallParser;
use serde_json::Value;

pub struct DefaultToolParser;

impl DefaultToolParser {
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

    fn extract_and_push_call(json_str: &str, calls: &mut Vec<(String, Value)>) {
        if json_str.is_empty() {
            return;
        }

        match serde_json::from_str::<Value>(json_str) {
            Ok(val) => {
                if val.is_object() {
                    if let (Some(name), Some(args)) =
                        (val.get("name").and_then(|n| n.as_str()), val.get("args"))
                    {
                        calls.push((name.to_string(), args.clone()));
                    }
                } else if val.is_array() {
                    if let Some(arr) = val.as_array() {
                        for item in arr {
                            if let (Some(name), Some(args)) =
                                (item.get("name").and_then(|n| n.as_str()), item.get("args"))
                            {
                                calls.push((name.to_string(), args.clone()));
                            }
                        }
                    }
                }
            }
            Err(_) => {
                log::warn!("Failed to parse Tool JSON: {}", json_str);
            }
        }
    }
}

impl ToolCallParser for DefaultToolParser {
    fn parse(&self, text: &str) -> Vec<(String, Value)> {
        let mut calls = Vec::new();
        let tags = [
            ("[TOOL_CALL]", "[/TOOL_CALL]"),
            ("<tool_call>", "</tool_call>"),
            ("<tool>", "</tool>"),
            ("<function_call>", "</function_call>"),
        ];

        let mut found_with_tags = false;

        for (start_tag, end_tag) in tags {
            let mut current_text = text;
            while let Some(start) = current_text.find(start_tag) {
                found_with_tags = true;
                let content_start = start + start_tag.len();

                if let Some(end_offset) = current_text[content_start..].find(end_tag) {
                    let end = content_start + end_offset;
                    let raw_json_part = &current_text[content_start..end];
                    let clean_json = Self::clean_markdown_json(raw_json_part);

                    Self::extract_and_push_call(clean_json, &mut calls);
                    current_text = &current_text[end + end_tag.len()..];
                } else {
                    let raw_json_part = &current_text[content_start..];
                    let clean_json = Self::clean_markdown_json(raw_json_part);
                    Self::extract_and_push_call(clean_json, &mut calls);
                    break;
                }
            }
        }

        if !found_with_tags {
            let mut current_text = text;
            while let Some(start) = current_text.find("```json") {
                let content_start = start + 7;
                if let Some(end_offset) = current_text[content_start..].find("```") {
                    let end = content_start + end_offset;
                    let clean_json = current_text[content_start..end].trim();
                    Self::extract_and_push_call(clean_json, &mut calls);
                    current_text = &current_text[end + 3..];
                } else {
                    break;
                }
            }
        }

        calls
    }
}
