#[cfg(test)]
mod tests {
    use ambi::llm::providers::openai_api::OpenAIEngineConfig;
    use ambi::{Agent, ChatPipeline, LLMEngineConfig};

    #[tokio::test]
    async fn test_ollama_local_api() {
        let base_url =
            std::env::var("TEST_BASE_URL").unwrap_or("https://api.openai.com/v1".to_string());
        let model_name = std::env::var("TEST_MODEL_NAME").unwrap_or("gpt-4o-mini".to_string());

        let cfg = OpenAIEngineConfig {
            api_key: "test".to_string(),
            base_url,
            model_name,
            temp: 0.7,
            top_p: 0.9,
        };

        let mut agent = Agent::make(LLMEngineConfig::OpenAI(cfg)).await.unwrap();

        let res = agent.chat("who are you").await.unwrap();

        println!("{}", res);
    }
}
