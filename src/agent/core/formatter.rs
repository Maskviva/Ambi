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

/// A stream formatter that intelligently parses and filters output tokens in real-time.
///
/// `TagStreamFormatter` is primarily used to intercept and hide internal model reasoning
/// (such as `<think>` blocks) and tool invocation tags (like `[TOOL_CALL]`) from the final
/// user-facing output stream. It maintains an internal buffer to handle tokens that are
/// split across network chunks.
///
/// # Examples
///
/// ```rust
/// use ambi::agent::core::formatter::TagStreamFormatter;
/// use ambi::agent::tool::StreamFormatter;
///
/// let mut formatter = TagStreamFormatter::new("[TOOL_CALL]", "[/TOOL_CALL]")
///     .set_max_buffer_size(8192);
///
/// // First push: Detected plain text, automatically output with [Content]: prefix.
/// // But because the end contains part of the tag "[TOOL", the main text was retained in the buffer waiting for closure.
/// let output1 = formatter.push("Here is the tool: [TOOL");
/// assert_eq!(output1, "[Content]: ");
///
/// // Second push: Tag closed, tool content hidden.
/// // Previously intercepted 'Here is the tool: ' has been safely released.
/// let output2 = formatter.push("_CALL]{\"name\":\"get_time\"}[/TOOL_CALL]");
/// assert_eq!(output2, "Here is the tool: ");
///
/// // Third push: ordinary text is passed through directly.
/// let output3 = formatter.push("Done.");
/// assert_eq!(output3, "Done.");
/// ```
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
    fn ends_with_partial_tag(&self, text: &str, tag: &str) -> bool {
        self.suffix_matches_tag_prefix(text, tag)
    }

    fn suffix_matches_tag_prefix(&self, text: &str, tag: &str) -> bool {
        // Check one by one whether the shortest prefix of the tag (length 1) to the full length matches the suffix of the text
        for len in 1..=tag.len() {
            let prefix = &tag[..len];
            if text.ends_with(prefix) {
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
                if let Some(idx) = self.buffer.find("<think>") {
                    let before = &self.buffer[..idx];
                    output.push_str(before);
                    output.push_str("[Thinking]:\n");
                    self.buffer = self.buffer[idx + 7..].to_string();
                    self.started = true;
                    continue;
                } else if self.buffer.contains(&self.start_tag) {
                    self.started = true;
                } else if self.suffix_matches_tag_prefix(&self.buffer, "<think>")
                    || self.suffix_matches_tag_prefix(&self.buffer, &self.start_tag)
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
