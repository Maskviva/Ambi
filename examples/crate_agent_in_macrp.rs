// Import necessary dependencies from the Ambi framework.
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::macros::{agent, tool};
use ambi::types::ToolErr;
use ambi::LLMEngineConfig;
use anyhow::Result;

//！ A simple arithmetic tool that returns the sum of two integers.
//！ The `#[tool]` macro automatically registers this function as an autonomous capability for the LLM.
//！ - `name`: The explicit tool identifier exposed to the model.
//！ - `timeout`: Forcefully aborts the execution if it exceeds 10 seconds.
//！ - `idempotent`: Marks the tool as safe to auto-retry upon failure.

/// Addition tool, returns the result of a and b
#[tool(name = "add", timeout = 10, idempotent)]
async fn add(a: i32, b: i32) -> Result<i32, ToolErr> {
    Ok(a + b)
}

/// The `#[agent]` macro automatically generates a builder pattern for this struct,
/// allowing seamless integration of registered tools (e.g., `AddTool`) into the agent.
#[agent(tools =[AddTool])]
pub struct DevAgent;

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Securely retrieve the API key from environment variables.
    let api_key = std::env::var("OPENAI_API_KEY")?;

    // Step 2: Configure the cloud-based LLM engine.
    // We use the OpenAI API configuration format with a temperature of 0.0 for deterministic outputs.
    let engine_config =
        LLMEngineConfig::OpenAI(OpenAIEngineConfig::create(api_key, "gpt-3.5-turbo").temp(0.0));

    // Step 3: Instantiate the agent using the generated builder pattern.
    // Thanks to the `#[agent]` macro, we can call `builder()` to construct the agent,
    // set a preamble (system prompt), and finalize via `build()`.
    let assistant = DevAgent::builder(engine_config)
        .preamble("You are an intelligent assistant and can use tools.")
        .build()
        .await?;

    // Step 4: Initiate a synchronous chat request to the LLM.
    // The agent will autonomously decide to invoke the `add` tool to fulfill the user's request.
    let reply = assistant.chat("What is 114514 plus 8080?").await?;

    // Step 5: Print the final response received from the model.
    println!("Response: {}", reply);
    Ok(())
}
