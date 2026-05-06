// You must enable the macro feature before starting (e.g., `cargo run --features ambi/macro`)
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::types::ToolErr;
use ambi::{tool, Agent, AgentState, ChatRunner, LLMEngineConfig};
use anyhow::Result;
use serde::Serialize;

/// Defines the structured output returned by the tool.
/// The LLM will receive this data serialized as a JSON payload.
#[derive(Serialize)]
pub struct WeatherOutput {
    temp: f32,
    condition: String,
}

/// The `#[tool]` macro automatically registers this function as an autonomous capability for the LLM.
/// - `name`: The explicit tool identifier exposed to the model.
/// - `timeout`: Forcefully aborts the execution if it exceeds 10 seconds.
/// - `idempotent`: Marks the tool as safe to auto-retry upon failure.
///
/// Note: The `Option<T>` type is automatically mapped to a non-required field in the JSON Schema,
/// enhancing the model's fault tolerance during argument generation.
#[tool(name = "check_city_weather", timeout = 10, idempotent)]
async fn get_weather(
    city: String,
    days: Option<u32>,
) -> core::result::Result<WeatherOutput, ToolErr> {
    let _query_days = days.unwrap_or(1);
    println!("Checking weather for {} ...", city);

    // Simulated API response
    Ok(WeatherOutput {
        temp: 22.5,
        condition: "Sunny".into(),
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Securely retrieve the API key from environment variables.
    // Fallback to a placeholder string if the environment variable is not set.
    let api_key =
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "your-default-api-key".to_string());

    // Step 2: Configure the cloud-based LLM engine.
    // We use the OpenAI API configuration format, which is highly compatible with
    // many backend providers (e.g., DeepSeek, Groq, vLLM, Ollama).
    let engine_config = LLMEngineConfig::OpenAI(OpenAIEngineConfig {
        api_key,
        base_url: "https://api.openai.com/v1".to_string(), // The base URL of the API endpoint.
        model_name: "gpt-4o-mini".to_string(),             // The specific model identifier.
        temp: 0.7,  // Controls randomness (higher values yield more creative outputs).
        top_p: 0.9, // Controls diversity via nucleus sampling.
    });

    // Step 3: Instantiate the ChatRunner.
    // This acts as the pipeline orchestrator responsible for managing the
    // ReAct loop (LLM <-> Tool interactions).
    let chat_runner = ChatRunner::default();

    // Step 4: Instantiate the Agent using the builder pattern.
    // Thanks to the zero-alias macro design, we can seamlessly mount the tool
    // by simply passing its original function name (`get_weather`).
    let agent = Agent::make(engine_config).await?.tool(GetWeatherTool)?;

    // Step 5: Initialize a thread-safe, shared agent state via the new_shared() convenience constructor.
    // `AgentState` is decoupled from the `Agent` itself. This architecture allows
    // a single read-only Agent instance to handle multiple concurrent conversations safely.
    let agent_state = AgentState::new_shared("session-id");

    // Step 6: Execute the chat pipeline.
    // The agent will autonomously decide to invoke the `check_city_weather` tool
    // to fulfill the user's request, process the tool's output, and return the final synthesis.
    let res = chat_runner
        .chat(&agent, &agent_state, "What's the weather in Beijing?")
        .await?;

    println!("Assistant: {}", res);

    Ok(())
}
