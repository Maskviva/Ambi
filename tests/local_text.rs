#[cfg(all(test, feature = "llama-cpp"))]
mod tests {
    use ambi::llm::providers::llama_cpp::LlamaEngineConfig;
    use ambi::{Agent, ChatPipeline, LLMEngineConfig};
    use std::io::Write;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_local_chat() {
        let model_path = std::env::var("TEST_MODEL_PATH").unwrap();

        let cfg = LlamaEngineConfig {
            model_path,
            max_tokens: 2048,
            buffer_size: 32,
            use_gpu: true,
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

        let mut agent = Agent::make(LLMEngineConfig::Llama(cfg)).await.unwrap();

        let mut res_stream = agent
            .chat_stream("afegrhtnmnd4aw684fv6e54sf1c35g4rv6e5rsf46ew54g1vw6e534")
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
