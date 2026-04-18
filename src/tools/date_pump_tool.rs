use crate::core::tool::{Tool, ToolDefinition, ToolErr};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PumpArgs {}

pub struct DatePumpTool;

#[async_trait]
impl Tool for DatePumpTool {
    const NAME: &'static str = "get_date";
    type Args = PumpArgs;
    type Output = String;

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "获取日期。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _arg: Self::Args) -> Result<Self::Output, ToolErr> {
        println!("\n 模型调用工具 DatePumpTool \n");

        let local_time = chrono::Local::now();
        let formatted_time = local_time.format("%Y-%m-%d %H:%M:%S").to_string();
        Ok(formatted_time)
    }
}
