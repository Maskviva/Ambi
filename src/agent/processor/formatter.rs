// src/agent/core/formatter.rs

use crate::types::StreamFormatter;

/// A passthrough formatter that performs zero modifications to the text.
pub struct PassThroughFormatter;
impl StreamFormatter for PassThroughFormatter {
    fn push(&mut self, token: &str) -> String {
        token.to_string()
    }
    fn flush(&mut self) -> String {
        String::new()
    }
}

/// The framework's default standard stream formatter.
pub struct StandardStreamFormatter {
    buffer: String,
    in_tool_call: bool,
    started: bool,

    tool_start_tag: String,
    tool_end_tag: String,

    think_start_tag: String,
    think_end_tag: String,

    max_buffer_size: usize,
}

impl StandardStreamFormatter {
    /// Initializes the formatter with tags to intercept and hide.
    pub fn new(tool_start: &str, tool_end: &str, think_start: &str, think_end: &str) -> Self {
        Self {
            buffer: String::new(),
            in_tool_call: false,
            started: false,
            tool_start_tag: tool_start.to_string(),
            tool_end_tag: tool_end.to_string(),

            think_start_tag: if think_start.is_empty() {
                "<think>".to_string()
            } else {
                think_start.to_string()
            },
            think_end_tag: if think_end.is_empty() {
                "</think>".to_string()
            } else {
                think_end.to_string()
            },
            max_buffer_size: 8192,
        }
    }

    /// Defines a maximum buffer limit to prevent Out-Of-Memory exploits on stream chunks.
    pub fn set_max_buffer_size(mut self, max_buffer_size: usize) -> Self {
        self.max_buffer_size = max_buffer_size;
        self
    }

    fn process_text(&self, text: &str) -> String {
        text.replace(&self.think_end_tag, "\n\n[Content]: ")
    }

    fn suffix_matches_tag_prefix(&self, text: &str, tag: &str) -> bool {
        if tag.is_empty() {
            return false;
        }
        for len in 1..=tag.len() {
            let prefix = &tag[..len];
            if text.ends_with(prefix) {
                return true;
            }
        }
        false
    }
}

impl StreamFormatter for StandardStreamFormatter {
    fn push(&mut self, token: &str) -> String {
        self.buffer.push_str(token);

        if self.buffer.len() > self.max_buffer_size {
            self.buffer.clear();
            self.in_tool_call = false;
            self.started = true;

            log::error!(
                "Formatter buffer overflow (>{}). Discarding buffered data to prevent OOM.",
                self.max_buffer_size
            );
            return String::new();
        }

        let mut output = String::new();

        loop {
            if !self.started {
                let trimmed = self.buffer.trim_start();
                if let Some(idx) = self.buffer.find(&self.think_start_tag) {
                    let before = &self.buffer[..idx];
                    output.push_str(before);
                    output.push_str("[Thinking]:\n");
                    self.buffer = self.buffer[idx + self.think_start_tag.len()..].to_string();
                    self.started = true;
                    continue;
                } else if self.buffer.contains(&self.tool_start_tag) {
                    self.started = true;
                } else if self.suffix_matches_tag_prefix(&self.buffer, &self.think_start_tag) {
                    break;
                } else if self.suffix_matches_tag_prefix(&self.buffer, &self.tool_start_tag) {
                    if !trimmed.is_empty() {
                        output.push_str("[Content]: ");
                        self.started = true;
                    }
                    break;
                } else if !trimmed.is_empty() {
                    output.push_str("[Content]: ");
                    self.started = true;
                } else {
                    break;
                }
            }

            if self.in_tool_call {
                if let Some(end_idx) = self.buffer.find(&self.tool_end_tag) {
                    self.buffer = self.buffer[end_idx + self.tool_end_tag.len()..].to_string();
                    self.in_tool_call = false;
                    continue;
                }
                break;
            } else {
                if let Some(start_idx) = self.buffer.find(&self.tool_start_tag) {
                    let before = self.buffer[..start_idx].to_string();
                    output.push_str(&self.process_text(&before));
                    self.buffer = self.buffer[start_idx + self.tool_start_tag.len()..].to_string();
                    self.in_tool_call = true;
                    continue;
                }
                if self.suffix_matches_tag_prefix(&self.buffer, &self.tool_start_tag)
                    || self.suffix_matches_tag_prefix(&self.buffer, &self.think_end_tag)
                    || self.suffix_matches_tag_prefix(&self.buffer, &self.think_start_tag)
                {
                    break;
                }
                let safe_text = self.buffer.clone();
                output.push_str(&self.process_text(&safe_text));
                self.buffer.clear();
                break;
            }
        }
        output
    }

    fn flush(&mut self) -> String {
        let res = self.process_text(&self.buffer);
        self.buffer.clear();
        res
    }
}
