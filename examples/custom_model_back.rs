// Import traits and types required for implementing a custom LLM backend.
use ambi::error::Result;
use ambi::llm::LLMEngineTrait;
use ambi::types::LLMRequest;
use ambi::Agent;
use async_trait::async_trait;

// Step 1: Define the configuration and state structure for your custom engine.
// This struct will hold API keys, client instances, or any proprietary state.
struct MyCompanyEngine {
    _api_key: String,
}

// Step 2: Implement the `LLMEngineTrait` to integrate your engine into Ambi.
// The `#[async_trait]` macro is required because trait methods contain async operations.
#[async_trait]
impl LLMEngineTrait for MyCompanyEngine {
    // Implement the synchronous/non-streaming chat method.
    async fn chat(&mut self, _request: LLMRequest) -> Result<String> {
        // Here, you would typically make an HTTP request using `request.formatted_prompt`.
        // For demonstration, we simply return a hardcoded string.
        Ok("I am a custom AI assistant.".to_string())
    }

    // Implement the asynchronous/streaming chat method.
    async fn chat_stream(
        &mut self,
        _request: LLMRequest,
        tx: tokio::sync::mpsc::Sender<Result<String>>,
    ) {
        // Here, you would stream data from your backend and send chunks through `tx`.
        // Simulating a stream:
        let _ = tx.send(Ok("Streaming ".to_string())).await;
        let _ = tx.send(Ok("response...".to_string())).await;
    }

    // Implement the method to clear internal session contexts or KV caches if applicable.
    fn reset_context(&mut self) {
        // Clear any internal states here.
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Step 3: Define the system prompt.
    let system_prompt = "You are a helpful and harmless AI assistant.";

    // Step 4: Instantiate your custom engine and box it as a dynamically dispatched trait object.
    let my_engine = Box::new(MyCompanyEngine {
        _api_key: "secret-api-key".to_string(),
    });

    // Step 5: Mount your custom engine to the Agent using `Agent::with_custom_engine`.
    let mut _agent = Agent::with_custom_engine(my_engine)?.preamble(system_prompt);

    // The agent is now fully functional and backed by your proprietary model engine.
    Ok(())
}
