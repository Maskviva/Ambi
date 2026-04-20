use crate::agent::tool::ToolErr;
use crate::agent::DynTool;
use crate::ToolDefinition;

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
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

    pub fn parse_tool_calls(text: &str) -> Vec<(String, Value)> {
        let mut calls = Vec::new();
        let mut current_text = text;

        while let Some(start) = current_text.find("[TOOL_CALL]") {
            let json_start = start + 11;

            if let Some(end_offset) = current_text[json_start..].find("[/TOOL_CALL]") {
                let end = json_start + end_offset;
                let mut json_part = current_text[json_start..end].trim();

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
                        calls.push((name.to_string(), args.clone()));
                    }
                } else {
                    log::warn!("Failed to parse TOOL_CALL JSON: {}", json_part);
                }

                current_text = &current_text[end + 12..];
            } else {
                break;
            }
        }
        calls
    }

    pub async fn run_tool(
        tool_map: &HashMap<String, Arc<dyn DynTool>>,
        name: String,
        args: &Value,
    ) -> Result<String, ToolErr> {
        let tool = tool_map
            .get(&name)
            .ok_or_else(|| ToolErr(format!("Tool {} not found", name)))?;

        let def = tool.definition();
        let timeout_duration = Duration::from_secs(def.timeout_secs.unwrap_or(15));
        let mut retries = def.max_retries.unwrap_or(3);

        loop {
            match timeout(timeout_duration, tool.call_json(args.clone())).await {
                Ok(Ok(result)) => {
                    return serde_json::to_string(&result).map_err(|e| ToolErr(e.to_string()));
                }
                Ok(Err(e)) => {
                    retries = retries.saturating_sub(1);
                    if retries == 0 {
                        return Err(e);
                    }
                    log::warn!(
                        "Tool '{}' execution error, retrying... ({} attempts remaining)",
                        name,
                        retries
                    );
                    sleep(Duration::from_millis(500)).await;
                }
                Err(_) => {
                    retries = retries.saturating_sub(1);
                    if retries == 0 {
                        return Err(ToolErr(format!(
                            "Tool '{}' execution timed out ({}s)",
                            name,
                            timeout_duration.as_secs()
                        )));
                    }
                    log::warn!(
                        "Tool '{}' execution timed out, retrying... ({} attempts remaining)",
                        name,
                        retries
                    );
                }
            }
        }
    }
}
