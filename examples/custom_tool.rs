use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::types::{Tool, ToolDefinition, ToolErr};
use ambi::{Agent, AgentState, ChatRunner, LLMEngineConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// 1. Define the input arguments structure
#[derive(Deserialize)]
struct CalculatorArgs {
    a: f64,
    b: f64,
    operation: String,
}

// 2. Define the output result structure
#[derive(Serialize)]
struct CalculatorResult {
    result: f64,
}

// 3. Create the Tool struct
struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    const NAME: &'static str = "calculator";
    type Args = CalculatorArgs;
    type Output = CalculatorResult;

    // Provide the JSON Schema definition for the LLM
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Performs basic mathematical operations (add, subtract, multiply, divide)."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": { "type": "number" },
                    "b": { "type": "number" },
                    "operation": { "type": "string", "enum":["add", "subtract", "multiply", "divide"] }
                },
                "required":["a", "b", "operation"]
            }),
            timeout_secs: Some(5),
            max_retries: Some(1),
            is_idempotent: true, // Math is safe to retry
        }
    }

    // The actual execution logic
    async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr> {
        let res = match args.operation.as_str() {
            "add" => args.a + args.b,
            "subtract" => args.a - args.b,
            "multiply" => args.a * args.b,
            "divide" => {
                if args.b == 0.0 {
                    return Err(ToolErr("Division by zero is not allowed".to_string()));
                }
                args.a / args.b
            }
            _ => return Err(ToolErr("Unknown operation".to_string())),
        };

        Ok(CalculatorResult { result: res })
    }
}

#[tokio::main]
async fn main() -> ambi::error::Result<()> {
    let config = OpenAIEngineConfig {
        /* API configuration... */
        api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
        base_url: "https://api.openai.com/v1".to_string(),
        model_name: "gpt-4o-mini".to_string(),
        temp: 0.1,
        top_p: 0.9,
    };

    // 4. Register the tool into the Agent
    let agent = Agent::make(LLMEngineConfig::OpenAI(config))
        .await?
        .preamble("You are a mathematical assistant. You MUST use the calculator tool to solve math problems.")
        .tool(CalculatorTool)? // <-- Injection
        .with_standard_formatting();

    let state = Arc::new(RwLock::new(AgentState::new()));
    let runner = ChatRunner;

    println!("User: What is 12345 multiplied by 67890?");

    // The agent will automatically call the Rust tool, get the answer, and reply!
    let response = runner
        .chat(&agent, &state, "What is 12345 multiplied by 67890?")
        .await?;
    println!("Assistant: {}", response);

    Ok(())
}
