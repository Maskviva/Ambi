#[derive(Default)]
pub struct StreamFormatter {
    buffer: String,
    in_tool_call: bool,
    started: bool,
}

impl StreamFormatter {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            in_tool_call: false,
            started: false,
        }
    }

    pub fn push(&mut self, token: &str) -> String {
        self.buffer.push_str(token);
        let mut output = String::new();

        loop {
            if !self.started {
                if self.buffer.starts_with("<think>") {
                    output.push_str("[Thinking]:\n");
                    self.buffer = self.buffer[7..].to_string();
                    self.started = true;
                    continue;
                } else if self.buffer.starts_with("[TOOL_CALL]") {
                    self.started = true;
                } else if self.is_partial_match(&self.buffer, "<think>")
                    || self.is_partial_match(&self.buffer, "[TOOL_CALL]")
                {
                    break;
                } else {
                    output.push_str("[Content]: ");
                    self.started = true;
                }
            }

            if self.in_tool_call {
                if let Some(end_idx) = self.buffer.find("[/TOOL_CALL]") {
                    self.buffer = self.buffer[end_idx + 12..].to_string();
                    self.in_tool_call = false;
                    continue;
                }
                break;
            } else {
                if let Some(start_idx) = self.buffer.find("[TOOL_CALL]") {
                    let before = self.buffer[..start_idx].to_string();
                    output.push_str(&self.process_text(&before));
                    self.buffer = self.buffer[start_idx + 11..].to_string();
                    self.in_tool_call = true;
                    continue;
                }

                if self.ends_with_partial_tag(&self.buffer, "[TOOL_CALL]")
                    || self.ends_with_partial_tag(&self.buffer, "</think>")
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

    pub fn flush(&mut self) -> String {
        let res = self.process_text(&self.buffer);
        self.buffer.clear();
        res
    }

    fn process_text(&self, text: &str) -> String {
        text.replace("</think>", "\n\n[Content]: ")
    }

    fn is_partial_match(&self, text: &str, tag: &str) -> bool {
        tag.starts_with(text)
    }

    fn ends_with_partial_tag(&self, text: &str, tag: &str) -> bool {
        for i in 1..tag.len() {
            if text.ends_with(&tag[..i]) {
                return true;
            }
        }
        false
    }
}
