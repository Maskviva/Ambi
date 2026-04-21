use ambi::llm::handler::LLMRequest;
use ambi::{Agent, LLMEngineTrait};
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

// Step 1: Developers define the configurations and structures they want themselves
struct MyCompanyEngine {
    api_key: String,
    // ... Model's exclusive fields
}

// Step 2: Implement the model's Trait
#[async_trait]
impl LLMEngineTrait for MyCompanyEngine {
    async fn chat(&mut self, request: LLMRequest) -> Result<String> {
        Ok("Im a AI assistant".to_string())
    }

    async fn chat_stream(&mut self, request: LLMRequest, tx: tokio::sync::mpsc::Sender<Result<String, anyhow::Error>>) {
        // ... Stream processing
    }

    fn reset_context(&mut self) {}
}

#[tokio::main]
async fn main() -> Result<()> {
    // Step 3: Instantiate the engine
    let my_engine = Box::new(MyCompanyEngine {
        api_key: "secret".to_string(),
    });

    // Step 4: Mount to your Agent
    let mut agent = Agent::with_custom_engine(my_engine)?.preamble("who are you");

    let _ = agent.chat("你好").await;

    Ok(())
}
