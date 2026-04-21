use anyhow::Result;
use std::io::Write;
use tokio_stream::StreamExt;

use ambi::llm::providers::llama_cpp::LlamaEngineConfig;
use ambi::llm::ChatTemplateType;
use ambi::LLMEngineConfig;
use ambi::{Agent, ChatPipeline};

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
    // Step 1: Initialize the logging system.
    init_logger();

    // Step 2: Define the system prompt.
    let system_prompt = "You are a helpful and harmless AI assistant.";

    // Step 3: Configure the Local Llama Engine parameters.
    let engine_config = LLMEngineConfig::Llama(LlamaEngineConfig {
        model_path: "C:/your-dir-path/model.gguf".to_string(), // Absolute path to the local GGUF model
        max_tokens: 4096,     // Maximum number of tokens to generate
        buffer_size: 32,      // Output buffer size
        use_gpu: true,        // Enable GPU acceleration
        n_gpu_layers: 100,    // Number of layers to offload to the GPU
        n_ctx: 4096,          // Context window size
        n_tokens: 4096,       // Batch size for prompt processing
        n_seq_max: 1,         // Maximum concurrent sequences
        penalty_last_n: 64,   // Range of tokens to consider for repetition penalty
        penalty_repeat: 1.1,  // Repetition penalty coefficient
        penalty_freq: 0.0,    // Frequency penalty coefficient
        penalty_present: 0.0, // Presence penalty coefficient
        temp: 0.7,            // Temperature for randomness control
        top_p: 0.9,           // Top-P sampling threshold
        seed: 299792458,      // Random seed for deterministic outputs
        min_keep: 1,
    });

    // Step 4: Instantiate the Agent.
    let mut agent = Agent::make(engine_config)
        .await?
        .template(ChatTemplateType::Chatml)
        .preamble(system_prompt);

    // Step 5: Initiate the chat stream and handle the output.
    let mut res_stream = agent
        .chat_stream("Who are you and what can you do?")
        .await
        .map_err(|_| anyhow::anyhow!("Failed to create chat stream"))?;

    while let Some(chunk) = res_stream.next().await {
        if let Ok(text) = chunk {
            print!("{}", text);
            let _ = std::io::stdout().flush();
        }
    }

    println!();
    Ok(())
}
