use ambi::llm::providers::openai_api::config::OpenAIEngineConfig;
use ambi::macros::{agent, tool};
use ambi::types::ToolErr;
use ambi::LLMEngineConfig;
use anyhow::Result;

/// 加法工具， 返回 a + b 的结果
#[tool(name = "add", timeout = 10, idempotent)]
async fn add(a: i32, b: i32) -> Result<i32, ToolErr> {
    Ok(a + b)
}

#[agent(tools =[AddTool])]
pub struct DevAgent;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")?;

    let engine_config =
        LLMEngineConfig::OpenAI(OpenAIEngineConfig::create(api_key, "gpt-3.5-turbo").temp(0.0));

    let assistant = DevAgent::builder(engine_config)
        .preamble("你是一个智能助手，会使用工具。")
        .build()
        .await?;

    let reply = assistant
        .chat("你试试你的add工具能不能用，我在调试。")
        .await?;

    println!("Response: {}", reply);
    Ok(())
}
