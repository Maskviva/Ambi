// Import the necessary configuration structs and the ChatTemplate types.
use ambi::llm::ChatTemplate;
use ambi::types::config::OpenAIEngineConfig;
use ambi::{Agent, LLMEngineConfig};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Set up a standard engine configuration.
    // This represents the connection to your inference backend.
    let engine_config = LLMEngineConfig::OpenAI(OpenAIEngineConfig {
        api_key: "mock-key".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        model_name: "gpt-4o-mini".to_string(),
        temp: 0.7,
        top_p: 0.9,
    });

    // Step 2: Define a fully custom ChatTemplate.
    // This is crucial when using proprietary or fine-tuned models that require unique conversational markers.
    let custom_tpl = ChatTemplate {
        system_prefix: "<|SYS_START|>\n".to_string(), // Prefix added before the system prompt.
        system_suffix: "\n<|SYS_END|>\n\n".to_string(), // Suffix added after the system prompt.
        user_prefix: "<|HUMAN|>: ".to_string(),       // Prefix added before every user message.
        user_suffix: "\n".to_string(),                // Suffix added after every user message.
        assistant_prefix: "<|BOT|>: ".to_string(), // Prefix indicating the assistant's turn to generate.
        assistant_suffix: "<|EOT|>\n".to_string(), // Suffix marking the end of the assistant's response.
        tool_prefix: "<|TOOL_EXECUTION|>\n".to_string(), // Prefix wrapping tool execution outputs.
        tool_suffix: "\n<|END_EXECUTION|>\n".to_string(), // Suffix wrapping tool execution outputs.
    };

    // Step 3: Instantiate the Agent and inject the custom template.
    // The framework will use these exact markers to assemble the prompt with zero-copy overhead.
    let mut _agent = Agent::make(engine_config).await?.template(custom_tpl); // Pass the custom struct directly into the builder.

    // Note: The agent is now ready to chat using your custom formatting rules.
    Ok(())
}
