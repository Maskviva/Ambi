use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::io::Write;
use tokio_stream::StreamExt;

use ambi::core::llm::chat_template::ChatTemplateType;
use ambi::core::tool::ToolErr;
use ambi::{Agent, Tool, ToolDefinition};
use ambi::{EngineConfig, OpenAIEngineConfig};

// ==========================================
// Part 1: Custom Tool Definition
// ==========================================

// 1. Define the arguments required by the tool.
// Since this tool only retrieves the current time, no external arguments are needed.
#[derive(Deserialize)]
pub struct PumpArgs {}

// 2. Define the tool struct.
pub struct DatePumpTool;

// 3. Implement the `Tool` trait to make it recognizable and executable by the Agent.
#[async_trait]
impl Tool for DatePumpTool {
    // The unique identifier used by the LLM to trigger this tool.
    const NAME: &'static str = "get_date";
    type Args = PumpArgs;
    type Output = String;

    // Describe the tool's function and parameter schema (JSON Schema) to the LLM.
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Get the current local date and time.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    // The actual execution logic when the tool is invoked.
    async fn call(&self, _arg: Self::Args) -> Result<Self::Output, ToolErr> {
        println!("\n[System] Tool 'DatePumpTool' invoked by the LLM...\n");
        let local_time = chrono::Local::now();
        Ok(local_time.format("%Y-%m-%d %H:%M:%S").to_string())
    }
}

// ==========================================
// Part 2: Application Entry Point
// ==========================================

fn init_logger() {
    use simplelog::*;
    let _ = TermLogger::init(
        LevelFilter::Info, // Lower the log level to Info for a cleaner output in this demo
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Initialize the logging system.
    init_logger();

    // Step 2: Set the system prompt.
    // Crucially, inform the LLM that it has the capability to use tools.
    let system_prompt = "You are a helpful AI assistant with tool-calling capabilities.";
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test-key".to_string());

    // Step 3: Configure the Engine (using the Cloud engine as an example).
    let engine_config = EngineConfig {
        llama: None,
        open_ai: Some(OpenAIEngineConfig {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model_name: "gpt-4o-mini".to_string(),
            temp: 0.7,
            top_p: 0.9,
        }),
    };

    // Step 4: Instantiate the Agent and **Mount the Custom Tool**.
    let mut agent = Agent::make(engine_config)?
        .template(ChatTemplateType::Chatml)
        .preamble(system_prompt)
        .tool(DatePumpTool)?; // <-- Injecting the custom tool here

    // Step 5: Ask a question that explicitly requires the tool to answer correctly.
    let mut res_stream = agent
        .chat_stream("What is the current local date and time?")
        .await
        .map_err(|_| anyhow::anyhow!("Failed to create chat stream"))?;

    // Step 6: Consume the stream.
    // The LLM will automatically halt generation, execute the tool, and resume with the final answer.
    while let Some(chunk) = res_stream.next().await {
        if let Ok(text) = chunk {
            print!("{}", text);
            let _ = std::io::stdout().flush();
        }
    }

    println!();
    Ok(())
}
