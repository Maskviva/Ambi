use crate::agent::tool::StreamFormatter;

pub struct PassThroughFormatter;
impl StreamFormatter for PassThroughFormatter {
    fn push(&mut self, token: &str) -> String {
        token.to_string()
    }
    fn flush(&mut self) -> String {
        String::new()
    }
}

pub struct TagStreamFormatter {
    buffer: String,
    in_tool_call: bool,
    started: bool,
    start_tag: String,
    end_tag: String,
    max_buffer_size: usize,
}

impl TagStreamFormatter {
    pub fn new(start_tag: &str, end_tag: &str) -> Self {
        Self {
            buffer: String::new(),
            in_tool_call: false,
            started: false,
            start_tag: start_tag.to_string(),
            end_tag: end_tag.to_string(),
            max_buffer_size: 8192,
        }
    }

    pub fn set_max_buffer_size(mut self, max_buffer_size: usize) -> Self {
        self.max_buffer_size = max_buffer_size;
        self
    }

    fn process_text(&self, text: &str) -> String {
        text.replace("</think>", "\n\n[Content]: ")
    }
    fn is_partial_match(&self, text: &str, tag: &str) -> bool {
        tag.starts_with(text)
    }
    fn ends_with_partial_tag(&self, text: &str, tag: &str) -> bool {
        for (i, _) in tag.char_indices().skip(1) {
            if text.ends_with(&tag[..i]) {
                return true;
            }
        }
        false
    }
}

impl StreamFormatter for TagStreamFormatter {
    fn push(&mut self, token: &str) -> String {
        self.buffer.push_str(token);

        if self.buffer.len() > self.max_buffer_size {
            log::error!(
                "Formatter buffer overflow (>{}). Force flushing to prevent OOM.",
                self.max_buffer_size
            );
            self.buffer.clear();
            self.in_tool_call = false;
            self.started = true;

            return "\n\n[Error: Tool output exceeded safety limits and was truncated by the system]\n\n".to_string();
        }

        let mut output = String::new();

        loop {
            if !self.started {
                let trimmed = self.buffer.trim_start();
                if let Some(idx) = self.buffer.find("<think>") {
                    let before = &self.buffer[..idx];
                    output.push_str(before);
                    output.push_str("[Thinking]:\n");
                    self.buffer = self.buffer[idx + 7..].to_string();
                    self.started = true;
                    continue;
                } else if self.buffer.contains(&self.start_tag) {
                    self.started = true;
                } else if self.is_partial_match(trimmed, "<think>")
                    || self.is_partial_match(trimmed, &self.start_tag)
                {
                    break;
                } else if !trimmed.is_empty() {
                    output.push_str("[Content]: ");
                    self.started = true;
                } else {
                    break;
                }
            }

            if self.in_tool_call {
                if let Some(end_idx) = self.buffer.find(&self.end_tag) {
                    self.buffer = self.buffer[end_idx + self.end_tag.len()..].to_string();
                    self.in_tool_call = false;
                    continue;
                }
                break;
            } else {
                if let Some(start_idx) = self.buffer.find(&self.start_tag) {
                    let before = self.buffer[..start_idx].to_string();
                    output.push_str(&self.process_text(&before));
                    self.buffer = self.buffer[start_idx + self.start_tag.len()..].to_string();
                    self.in_tool_call = true;
                    continue;
                }
                if self.ends_with_partial_tag(&self.buffer, &self.start_tag)
                    || self.ends_with_partial_tag(&self.buffer, "</think>")
                    || self.ends_with_partial_tag(&self.buffer, "<think>")
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
