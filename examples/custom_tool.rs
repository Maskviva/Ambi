use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use ambi::agent::tool::ToolErr;
use ambi::agent::{Tool, ToolDefinition};
use ambi::llm::ChatTemplateType;
use ambi::types::config::OpenAIEngineConfig;
use ambi::{Agent, AgentState};
use ambi::{ChatRunner, LLMEngineConfig};

// Step 1: Define the arguments structure for the tool.
// These parameters will be populated by the LLM when it decides to call the tool.
// Since we only need the current time, this struct is intentionally empty.
#[derive(Deserialize)]
pub struct PumpArgs {}

// Step 2: Define the empty struct representing the Tool itself.
pub struct DatePumpTool;

// Step 3: Implement the `Tool` trait to make it executable by the framework.
#[async_trait]
impl Tool for DatePumpTool {
    // Provide a unique, programmatic name for the LLM to invoke.
    const NAME: &'static str = "get_date";
    type Args = PumpArgs;
    type Output = String;

    // Provide the JSON Schema definition that tells the LLM what this tool does and how to use it.
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Get the current local date and time.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            // Optionally override the default timeout and retry logic for this specific tool.
            timeout_secs: Some(10),
            max_retries: Some(3),
            is_idempotent: false,
        }
    }

    // Define the actual Rust code to execute when the tool is called.
    async fn call(&self, _arg: Self::Args) -> Result<Self::Output, ToolErr> {
        println!("\n[System] Tool 'DatePumpTool' invoked by the LLM...\n");
        let local_time = chrono::Local::now();
        Ok(local_time.format("%Y-%m-%d %H:%M:%S").to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Step 4: Instruct the LLM in the system prompt that it has tool-calling capabilities.
    let system_prompt = "You are a helpful AI assistant with tool-calling capabilities.";
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test-key".to_string());

    // Step 5: Configure the engine.
    let engine_config = LLMEngineConfig::OpenAI(OpenAIEngineConfig {
        api_key,
        base_url: "https://api.openai.com/v1".to_string(),
        model_name: "gpt-4o-mini".to_string(),
        temp: 0.7,
        top_p: 0.9,
    });

    // Step 6: Instantiate the ChatRunner. This is used to distinguish which `ChatRunner` it comes from.
    let chat_runner = ChatRunner;

    // Step 7: Create the Agent and mount the custom tool using `.tool()`.
    let agent = Agent::make(engine_config)
        .await?
        .template(ChatTemplateType::Chatml)
        .preamble(system_prompt)
        .tool(DatePumpTool)?;

    // Step 8: Initialize the agent state. The state will be stored here.
    let agent_state = Arc::new(Mutex::new(AgentState::new()));

    // Step 9: Ask a question that requires real-time data to trigger the tool automatically.
    let res = ChatRunner::chat(
        &chat_runner,
        &agent,
        &agent_state,
        "What is the current local date and time?",
    )
    .await?;

    // Step 10: Print the final answer synthesized by the LLM after observing the tool's output.
    print!("{}", res);

    Ok(())
}
