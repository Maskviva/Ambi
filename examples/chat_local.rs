use anyhow::Result;

// Import necessary configurations and traits for the local Llama engine.
use ambi::llm::ChatTemplateType;
use ambi::types::config::LlamaEngineConfig;
use ambi::Agent;
use ambi::{ChatRunner, LLMEngineConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Define the system prompt to set the AI's behavior.
    let system_prompt = "You are a helpful and harmless AI assistant.";

    // Step 2: Configure the local Llama engine parameters.
    // This requires a local GGUF model file and configures hardware acceleration.
    let engine_config = LLMEngineConfig::Llama(LlamaEngineConfig {
        model_path: "C:/your-dir-path/model.gguf".to_string(), // Absolute path to the local GGUF model file.
        max_tokens: 4096, // Maximum number of tokens the model can generate in a single response.
        buffer_size: 32,  // Size of the output buffer for token decoding.
        use_gpu: true,    // Enable GPU acceleration for faster inference.
        n_gpu_layers: 100, // Number of layers to offload to the GPU (100 usually means all layers).
        n_ctx: 4096,      // Maximum context window size (total tokens for prompt + response).
        n_tokens: 4096,   // Batch size for processing prompts.
        n_seq_max: 1,     // Maximum number of concurrent sequences.
        penalty_last_n: 64, // Number of recent tokens to consider for the repetition penalty.
        penalty_repeat: 1.1, // Coefficient for penalizing repeated tokens.
        penalty_freq: 0.0, // Coefficient for penalizing token frequency.
        penalty_present: 0.0, // Coefficient for penalizing token presence.
        temp: 0.7,        // Temperature for generation randomness.
        top_p: 0.9,       // Top-P (nucleus) sampling threshold.
        seed: 299792458,  // Random seed to ensure deterministic outputs.
        min_keep: 1,      // Minimum number of tokens to keep during sampling.
    });

    // Step 3: Instantiate the Agent.
    // Mount the local engine, apply the ChatML template, and set the system prompt.
    let mut agent = Agent::make(engine_config)
        .await?
        .template(ChatTemplateType::Chatml)
        .preamble(system_prompt);

    // Step 4: Send a chat message to the local model.
    // The framework handles prompt construction, context management, and inference.
    let res = ChatRunner::chat(&mut agent, "Who are you and what can you do?").await?;

    // Step 5: Output the result to the console.
    print!("{}", res);

    Ok(())
}
