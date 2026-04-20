use crate::agent::tool::ToolErr;
use crate::agent::DynTool;
use crate::ToolDefinition;

use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::{sleep, timeout};

pub struct ToolManager;

impl ToolManager {
    pub fn tool_prompt(tools: Vec<ToolDefinition>) -> String {
        let mut prompt = String::new();
        if !tools.is_empty() {
            let tools_json = serde_json::to_string(&tools).unwrap_or_default();
            prompt.push_str(&format!(
                "You can use tools. Call format:\n[TOOL_CALL]{{\"name\":\"tool_name\",\"args\":{{...}}}}[/TOOL_CALL]\nAvailable tools:\n{}",
                tools_json
            ));
        }
        prompt
    }

    pub fn parse_tool_call(text: &str) -> Option<(String, Value)> {
        if let Some(start) = text.find("[TOOL_CALL]") {
            let json_start = start + 11;

            if let Some(end_offset) = text[json_start..].find("[/TOOL_CALL]") {
                let end = json_start + end_offset;

                let mut json_part = text[json_start..end].trim();

                if json_part.starts_with("```json") {
                    json_part = json_part[7..].trim();
                } else if json_part.starts_with("```") {
                    json_part = json_part[3..].trim();
                }

                if json_part.ends_with("```") {
                    json_part = json_part[..json_part.len() - 3].trim();
                }

                if let Ok(val) = serde_json::from_str::<Value>(json_part) {
                    if let (Some(name), Some(args)) =
                        (val.get("name").and_then(|n| n.as_str()), val.get("args"))
                    {
                        return Some((name.to_string(), args.clone()));
                    }
                } else {
                    log::warn!("Failed to parse TOOL_CALL JSON: {}", json_part);
                }
            }
        }
        None
    }

    pub async fn run_tool(
        tool_map: &HashMap<String, Box<dyn DynTool>>,
        name: String,
        args: &Value,
    ) -> Result<String, ToolErr> {
        let tool = tool_map
            .get(&name)
            .ok_or_else(|| ToolErr(format!("Tool {} not found", name)))?;

        let mut retries = 3;

        loop {
            match timeout(Duration::from_secs(15), tool.call_json(args.clone())).await {
                Ok(Ok(result)) => {
                    return serde_json::to_string(&result).map_err(|e| ToolErr(e.to_string()));
                }
                Ok(Err(e)) => {
                    retries -= 1;
                    if retries == 0 {
                        return Err(e);
                    }
                    log::warn!("工具 '{}' 执行报错，重试... (剩余 {} 次)", name, retries);
                    sleep(Duration::from_millis(500)).await;
                }
                Err(_) => {
                    retries -= 1;
                    if retries == 0 {
                        return Err(ToolErr(format!("工具 '{}' 执行超时 (15s)", name)));
                    }
                    log::warn!("工具 '{}' 执行超时，重试... (剩余 {} 次)", name, retries);
                }
            }
        }
    }
}
