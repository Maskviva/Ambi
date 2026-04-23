// Import required configuration structs and traits.
use ambi::llm::ChatTemplateType;
use ambi::types::config::OpenAIEngineConfig;
use ambi::{Agent, LLMEngineConfig};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Configure the backend LLM engine.
    let engine_config = LLMEngineConfig::OpenAI(OpenAIEngineConfig {
        api_key: "mock-key".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        model_name: "gpt-4o-mini".to_string(),
        temp: 0.7,
        top_p: 0.9,
    });

    // Step 2: Instantiate the Agent and configure its memory management.
    let mut _agent = Agent::make(engine_config)
        .await?
        .template(ChatTemplateType::Chatml)
        // Step 3: Define the strict eviction strategy to trigger truncation early for testing.
        // Parameters: (keep_head_count, keep_tail_count, max_safe_tokens)
        .with_eviction_strategy(2, 2, 50)
        // Step 4: Inject a closure to handle messages that are evicted from the context window.
        // This is highly useful for archiving old conversations to a Vector Database for long-term RAG memory.
        .on_evict(|evicted_messages| {
            println!("\n[System Notification] Context window limit reached.");
            println!(
                "[Memory Manager] Evicting {} old messages...",
                evicted_messages.len()
            );

            // Step 5: Iterate through the evicted messages and simulate an archiving process.
            for (index, msg) in evicted_messages.iter().enumerate() {
                println!(
                    "  -> Archiving message #{}: {} bytes saved.",
                    index,
                    msg.text_len() // Using the zero-allocation length calculation method.
                );
            }
            println!("[Memory Manager] Archiving complete. Continuing conversation...\n");
        });

    println!("Agent initialized. Long conversation simulation ready.");

    // In a real scenario, you would repeatedly call `agent.chat(...)` here.
    // Once the total context exceeds 50 tokens, the `on_evict` closure will automatically trigger.

    Ok(())
}
