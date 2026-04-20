use ambi::llm::handler::LLMRequest;
use ambi::{Agent, LLMEngineTrait};
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

// 1. 开发者自己定义他们想要的配置和结构体
struct MyCompanyEngine {
    api_key: String,
    // ... 他们的专属字段
}

// 2. 开发者自己实现你的 Trait
#[async_trait]
impl LLMEngineTrait for MyCompanyEngine {
    async fn chat(&mut self, request: LLMRequest) -> Result<String> {
        // ... 他们自己的 HTTP 请求逻辑
        Ok("我们公司的私有模型回答".to_string())
    }

    async fn chat_stream(&mut self, request: LLMRequest, tx: Sender<String>) {
        // ... 他们的流式处理
    }

    fn reset_context(&mut self) {}
}

#[tokio::main]
async fn main() -> Result<()> {
    // 3. 实例化他们的引擎
    let my_engine = Box::new(MyCompanyEngine {
        api_key: "secret".to_string(),
    });

    // 4. 🌟 完美挂载到你的 Agent 上！
    let mut agent = Agent::with_custom_engine(my_engine)?.preamble("你是公司内部小助手");

    // 一切依然完美运转！工具调用、上下文管理全都能用！
    let _ = agent.chat("你好").await;

    Ok(())
}
