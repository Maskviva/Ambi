use crate::agent::tool::{DynTool, ToolDefinition, ToolErr};
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
                    log::warn!(
                        "Tool '{}' returned a deterministic error: {}. Aborting immediately.",
                        name,
                        e
                    );
                    return Err(e);
                }
                Err(_) => {
                    if !def.is_idempotent {
                        return Err(ToolErr(format!(
                            "Tool '{}' execution timed out ({}s). Not idempotent, aborting immediately.",
                            name, timeout_duration.as_secs()
                        )));
                    }

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
                    sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }
}
