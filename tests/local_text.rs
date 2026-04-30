#[cfg(all(test, feature = "llama-cpp"))]
mod tests {
    use ambi::llm::providers::llama_cpp::config::LlamaEngineConfig;
    use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig};
    use std::io::Write;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_local_chat() {
        let model_path = std::env::var("TEST_MODEL_PATH").unwrap();

        let cfg = LlamaEngineConfig {
            model_path,
            mmproj_path: None,
            integrated_vision: false,
            max_tokens: 2048,
            buffer_size: 32,
            use_gpu: false,
            n_gpu_layers: 99,
            n_ctx: 4096,
            n_tokens: 4096,
            n_seq_max: 1,
            penalty_last_n: 64,
            penalty_repeat: 1.1,
            penalty_freq: 0.0,
            penalty_present: 0.0,
            temp: 0.7,
            top_p: 0.9,
            seed: 299792458,
            min_keep: 1,
        };

        let chat_runner = ChatRunner;

        let agent = Agent::make(LLMEngineConfig::Llama(cfg)).await.unwrap();

        let agent_state = Arc::new(RwLock::new(AgentState::new()));

        let mut res_stream = chat_runner
            .chat_stream(&agent, &agent_state, "What is my name?")
            .await
            .unwrap();

        let mut res_buffe = String::new();

        while let Some(chunk) = res_stream.next().await {
            if let Ok(text) = chunk {
                print!("{}", text);
                res_buffe += &*text;
                let _ = std::io::stdout().flush();
            }
        }

        println!();

        let entropy = agent.evaluate_sentence_entropy(&*res_buffe).await.unwrap();

        println!("{}", entropy)
    }
}
