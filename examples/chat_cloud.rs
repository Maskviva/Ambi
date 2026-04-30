// Import necessary configurations and traits from the Ambi framework.
use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::types::ChatTemplateType;
use ambi::{Agent, ChatRunner};
use ambi::{AgentState, LLMEngineConfig};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Define the system prompt to set the persona and behavior of the AI assistant.
    let system_prompt = "You are a helpful and harmless AI assistant.";

    // Step 2: Retrieve the API key securely from environment variables.
    // Fallback to a default string if the environment variable is not set.
    let api_key =
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "your-default-api-key".to_string());

    // Step 3: Configure the cloud-based LLM engine.
    // Here we use the OpenAI API configuration format, which is compatible with many providers.
    let engine_config = LLMEngineConfig::OpenAI(OpenAIEngineConfig {
        api_key,
        base_url: "https://api.openai.com/v1".to_string(), // The base URL of the API endpoint.
        model_name: "gpt-4o-mini".to_string(),             // The specific model to use.
        temp: 0.7,  // Controls randomness (higher is more creative).
        top_p: 0.9, // Controls diversity via nucleus sampling.
    });

    // Step 4: Instantiate the ChatRunner. This is used to distinguish which `ChatRunner` it comes from.
    let chat_runner = ChatRunner;

    // Step 5: Instantiate the Agent using the builder pattern.
    // We pass the engine configuration, set the chat template, and inject the system prompt.
    let agent = Agent::make(engine_config)
        .await?
        .template(ChatTemplateType::Chatml)
        .preamble(system_prompt);

    // Step 6: Initialize the agent state. The state will be stored here.
    let agent_state = Arc::new(RwLock::new(AgentState::new()));

    // Step 7: Initiate a synchronous chat request to the LLM.
    // The agent will process the prompt, interact with the model, and return the final string.
    let res = chat_runner
        .chat(&agent, &agent_state, "Who are you and what can you do?")
        .await?;

    // Step 6: Print the final response received from the model.
    print!("{}", res);

    Ok(())
}
