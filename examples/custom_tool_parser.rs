// Import traits for parsing tool calls and formatting output streams.
use ambi::agent::tool::{StreamFormatter, ToolCallParser};
use ambi::llm::providers::openai_api::OpenAIEngineConfig;
use ambi::{Agent, LLMEngineConfig};
use anyhow::Result;
use serde_json::Value;

// Step 1: Define the struct for your custom XML syntax parser.
pub struct XmlToolParser;

// Step 2: Implement the `ToolCallParser` trait.
impl ToolCallParser for XmlToolParser {
    // Define how to instruct the LLM to format its tool calls.
    // This dynamically generates the tool instructions appended to the system prompt.
    fn format_instruction(&self, tools_json: &str) -> String {
        format!(
            "You can use tools. Call format:\n<tool_call name=\"tool_name\">{{\"args\":{{...}}}}</tool_call>\nAvailable tools:\n{}",
            tools_json
        )
    }

    // Define the logic to extract tool names and JSON arguments from the raw LLM output text.
    fn parse(&self, text: &str) -> Vec<(String, Value)> {
        let mut calls = Vec::new();
        let start_tag = "<tool_call name=\"";
        let end_tag = "</tool_call>";

        let mut current_text = text;

        // Iterate through the text to find all occurrences of the XML tags.
        while let Some(start_idx) = current_text.find(start_tag) {
            let name_start = start_idx + start_tag.len();

            // Extract the tool's name from the XML attribute.
            if let Some(quote_idx) = current_text[name_start..].find("\">") {
                let tool_name = &current_text[name_start..name_start + quote_idx];
                let content_start = name_start + quote_idx + 2;

                // Extract the JSON payload situated between the opening and closing tags.
                if let Some(end_idx) = current_text[content_start..].find(end_tag) {
                    let json_str = &current_text[content_start..content_start + end_idx];

                    // Attempt to parse the extracted string into a JSON Value.
                    if let Ok(args) = serde_json::from_str::<Value>(json_str) {
                        calls.push((tool_name.to_string(), args));
                    } else {
                        println!("Error: Invalid JSON arguments for tool '{}'", tool_name);
                    }
                    current_text = &current_text[content_start + end_idx + end_tag.len()..];
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        calls
    }

    // Provide a stream formatter.
    // Here we use the default pass-through formatter which prints everything as is.
    fn create_stream_formatter(&self) -> Box<dyn StreamFormatter> {
        Box::new(ambi::agent::core::formatter::PassThroughFormatter)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Step 3: Configure the backend engine.
    let engine_config = LLMEngineConfig::OpenAI(OpenAIEngineConfig {
        api_key: "mock-key".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        model_name: "gpt-4o-mini".to_string(),
        temp: 0.7,
        top_p: 0.9,
    });

    // Step 4: Instantiate the Agent and inject the custom tool parser using `.with_tool_parser()`.
    // The framework will now instruct the LLM to output XML and parse it accordingly.
    let mut _agent = Agent::make(engine_config)
        .await?
        .with_tool_parser(XmlToolParser);

    Ok(())
}
