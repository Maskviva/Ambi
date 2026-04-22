#[cfg(all(test, feature = "llama-cpp"))]
mod tests {
    use ambi::llm::providers::llama_cpp::LlamaEngineConfig;
    use ambi::{Agent, ChatPipeline, LLMEngineConfig};

    #[tokio::test]
    async fn test_local_chat() {
        let model_path = std::env::var("TEST_MODEL_PATH").unwrap();

        let cfg = LlamaEngineConfig {
            model_path,
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
            temp: 0.1,
            top_p: 0.9,
            seed: 299792458,
            min_keep: 1,
        };

        let mut agent = Agent::make(LLMEngineConfig::Llama(cfg)).await.unwrap();

        let res = agent.chat("who are you").await.unwrap();

        println!("{}", res);
    }
}
