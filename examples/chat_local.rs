// Import necessary configurations and traits for the local Llama engine.
use ambi::llm::providers::llama_cpp::config::LlamaEngineConfig;
use ambi::types::ChatTemplateType;
use ambi::{Agent, AgentState};
use ambi::{ChatRunner, LLMEngineConfig};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Define the system prompt to set the AI's behavior.
    let system_prompt = "You are a helpful and harmless AI assistant.";

    // Step 2: Configure the local Llama engine parameters.
    // This requires a local GGUF model file and configures hardware acceleration.
    let engine_config = LLMEngineConfig::Llama(LlamaEngineConfig {
        model_path: "C:/your-dir-path/model.gguf".to_string(), // Absolute path to the local GGUF model file.
        mmproj_path: None, // Absolute path to the external vision projector model file.
        integrated_vision: false, // Indicates whether the main LLM has native, integrated vision capabilities.
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

    // Step 3: Instantiate the ChatRunner. This is used to distinguish which `ChatRunner` it comes from.
    let chat_runner = ChatRunner::default();

    // Step 4: Instantiate the Agent.
    // Mount the local engine, apply the ChatML template, and set the system prompt.
    let agent = Agent::make(engine_config)
        .await?
        .template(ChatTemplateType::Chatml)
        .preamble(system_prompt);

    // Step 5: Initialize a thread-safe, shared agent state via the new_shared() convenience constructor.
    let agent_state = AgentState::new_shared("session-id");

    // Step 6: Send a chat message to the local model.
    // The framework handles prompt construction, context management, and inference.
    let res = chat_runner
        .chat(&agent, &agent_state, "Who are you and what can you do?")
        .await?;

    // Step 7: Output the result to the console.
    print!("{}", res);

    Ok(())
}
