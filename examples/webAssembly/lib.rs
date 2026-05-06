//! WebAssembly bridge for the Ambi framework.
//!
//! This module exports a stateful `AmbiSession` class to JavaScript, allowing
//! front-end applications to maintain conversational memory across multiple calls
//! without re-initializing the LLM engine.

#[cfg(target_arch = "wasm32")]
pub mod wasm_api {
    use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
    use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use wasm_bindgen::prelude::*;

    /// A stateful WebAssembly wrapper for the Ambi Agent.
    ///
    /// By exporting this struct to JavaScript as a Class, we ensure that the
    /// `AgentState` (conversational memory) is preserved in the browser's memory
    /// across multiple chat interactions.
    #[wasm_bindgen]
    pub struct AmbiSession {
        /// The read-only orchestration blueprint (holds tools, config, and engine).
        agent: Agent,
        /// The mutable, persistent conversational history.
        state: Arc<RwLock<AgentState>>,
    }

    #[wasm_bindgen]
    impl AmbiSession {
        /// Asynchronously creates and initializes a new persistent chat session.
        ///
        /// In JavaScript, you invoke this via:
        /// `const session = await AmbiSession.create(baseUrl, modelName, apiKey);`
        pub async fn create(
            base_url: String,
            model_name: String,
            api_key: String,
        ) -> Result<AmbiSession, JsValue> {
            // 1. Configure the API connection parameters
            let config = OpenAIEngineConfig {
                api_key,
                base_url,
                model_name,
                temp: 0.7,
                top_p: 0.95,
            };

            // 2. Build the Agent
            // We use standard formatting to automatically hide `<think>` blocks or tool syntaxes.
            let agent = Agent::make(LLMEngineConfig::OpenAI(config))
                .await
                .map_err(|e| JsValue::from_str(&format!("Agent initialization error: {}", e)))?
                .preamble("You are a smart AI running directly inside a Web Browser using WebAssembly! Please remember my context if I tell you.");
            // .with_standard_formatting();

            // 3. Initialize the persistent memory state via the thread-safe new_shared() constructor
            let state = AgentState::new_shared("session_id");

            Ok(AmbiSession { agent, state })
        }

        /// Sends a prompt to the Agent, processes it through the pipeline, and
        /// appends both the user's prompt and the model's response to the history.
        ///
        /// In JavaScript, you invoke this via:
        /// `const reply = await session.chat("Hello!");`
        pub async fn chat(&self, prompt: String) -> Result<String, JsValue> {
            let runner = ChatRunner;

            // Reuse the existing `self.agent` and `self.state` to maintain context
            let response = runner
                .chat(&self.agent, &self.state, &prompt)
                .await
                .map_err(|e| JsValue::from_str(&format!("Chat execution error: {}", e)))?;

            Ok(response)
        }

        /// Sends a prompt and streams the response back through JavaScript callbacks.
        ///
        /// Each content chunk is forwarded to `on_chunk` as soon as it arrives from
        /// the LLM engine. When the stream completes, `on_done` is called with the
        /// assembled full response.
        ///
        /// In JavaScript, you invoke this via:
        /// ```js
        /// session.chat_stream(
        ///   "Hello!",
        ///   (chunk) => {
        ///     // append chunk to UI in real-time
        ///   },
        ///   (full)  => {
        ///     // re-enable the send button etc.
        ///   },
        /// );
        /// ```
        pub async fn chat_stream(
            &self,
            prompt: String,
            on_chunk: js_sys::Function,
            on_done: js_sys::Function,
        ) -> Result<(), JsValue> {
            use futures::StreamExt;

            let runner = ChatRunner;

            // Obtain the async stream of response chunks
            let mut stream = runner
                .chat_stream(&self.agent, &self.state, &prompt)
                .await
                .map_err(|e| JsValue::from_str(&format!("Chat stream error: {}", e)))?;

            let mut full_response = String::new();

            // Consume the stream one chunk at a time
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        full_response.push_str(&text);
                        // Forward each text fragment to the JS callback in real-time
                        let this = JsValue::null();
                        let _ = on_chunk.call1(&this, &JsValue::from_str(&text));
                    }
                    Err(e) => {
                        let this = JsValue::null();
                        let _ =
                            on_chunk.call1(&this, &JsValue::from_str(&format!("\n[Error: {}]", e)));
                    }
                }
            }

            // Signal completion and pass the complete response text
            let this = JsValue::null();
            let _ = on_done.call1(&this, &JsValue::from_str(&full_response));

            Ok(())
        }

        /// Manually clears the agent's short-term memory (context history) from JavaScript.
        pub async fn clear_memory(&self) {
            ChatRunner::clear_history(&self.agent, &mut *self.state.write().await);
        }

        // After completion, you need to install the official wasm packaging tool (if you haven't installed it) `cargo install wasm-pack`
        // After successful installation, execute `wasm-pack build --release --target web` in the project root directory
        // If nothing goes wrong, a `pkg` folder will be generated in your project root directory, which contains the compiled wasm files
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("You should set the build target to wasm32-unknown-unknown");
}
