use anyhow::Result;
use std::io::Write;
use tokio_stream::StreamExt;

use ambi::llm::chat_template::ChatTemplateType;
use ambi::Agent;
use ambi::{LLMEngineConfig, OpenAIEngineConfig};

// ==========================================
// Helper: Initialize Terminal Logger
// ==========================================
fn init_logger() {
    use simplelog::*;
    let _ = TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Initialize the logger to monitor the internal state of the framework.
    init_logger();

    // Step 2: Define the system prompt to set the Agent's persona and behavior.
    let system_prompt = "You are a helpful and harmless AI assistant.";

    // Step 3: Safely retrieve the API key from environment variables.
    // Avoid hardcoding sensitive credentials in your source code.
    let api_key =
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "your-default-api-key".to_string());

    // Step 4: Configure the LLM Engine.
    // Here we leave the `llama` configuration empty and specify `open_ai`.
    let engine_config = LLMEngineConfig::OpenAI(OpenAIEngineConfig {
        api_key,
        base_url: "https://api.openai.com/v1".to_string(), // Supports OpenAI-compatible APIs
        model_name: "gpt-4o-mini".to_string(),
        temp: 0.7,
        top_p: 0.9,
    });

    // Step 5: Instantiate the Agent using the Builder pattern.
    // Inject the dialogue template and system prompt.
    let mut agent = Agent::make(engine_config).await?
        .template(ChatTemplateType::Chatml)
        .preamble(system_prompt);

    // Step 6: Initiate an asynchronous streaming chat request.
    let mut res_stream = agent
        .chat_stream("Who are you and what can you do?")
        .await
        .map_err(|_| anyhow::anyhow!("Failed to create chat stream"))?;

    // Step 7: Consume the stream and print data chunks in real-time.
    while let Some(chunk) = res_stream.next().await {
        if let Ok(text) = chunk {
            print!("{}", text);
            let _ = std::io::stdout().flush(); // Ensure immediate terminal output
        }
    }

    println!();
    Ok(())
}
