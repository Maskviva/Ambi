#[cfg(test)]
mod tests {
    use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
    use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_ollama_local_api() {
        let base_url =
            std::env::var("TEST_BASE_URL").unwrap_or("https://api.openai.com/v1".to_string());
        let model_name = std::env::var("TEST_MODEL_NAME").unwrap_or("gpt-4o-mini".to_string());
        let api_key = std::env::var("TEST_API_KEY").unwrap_or("test".to_string());

        let cfg = OpenAIEngineConfig {
            api_key,
            base_url,
            model_name,
            temp: 0.7,
            top_p: 0.9,
        };

        let agent = Agent::make(LLMEngineConfig::OpenAI(cfg)).await.unwrap();
        let agent_state = Arc::new(RwLock::new(AgentState::new()));

        let res = ChatRunner::chat(&ChatRunner, &agent, &agent_state, "who are you")
            .await
            .unwrap();

        println!("{}", res);
    }
}
