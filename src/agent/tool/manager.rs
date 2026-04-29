// src/agent/tool/manager.rs

use crate::types::{DynTool, ToolErr};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// The static orchestrator for executing tools dynamically.
pub struct ToolManager;

impl ToolManager {
    /// Execute the specified tool call.
    /// Includes strict timeout control and retry mechanism.
    pub async fn run_tool(
        tool_map: &HashMap<String, Arc<dyn DynTool>>,
        name: String,
        args: &Value,
    ) -> Result<String, ToolErr> {
        let tool = tool_map
            .get(&name)
            .ok_or_else(|| ToolErr(format!("Tool '{}' not found in registered tools", name)))?;

        let def = tool.definition();

        // Mandatory framework-level timeout protection to prevent poorly written user tools from causing the entire Agent thread to hang
        let timeout_duration = Duration::from_secs(def.timeout_secs.unwrap_or(15));

        // Core Fix: Retry count for non-idempotent tools is forced to 0
        let mut retries = if def.is_idempotent {
            def.max_retries.unwrap_or(3)
        } else {
            0
        };

        loop {
            // All tools (whether idempotent or not) must be safely wrapped with a timeout
            match timeout(timeout_duration, tool.call_json(args.clone())).await {
                Ok(Ok(result)) => {
                    return serde_json::to_string(&result).map_err(|e| ToolErr(e.to_string()));
                }
                Ok(Err(e)) => {
                    // The tool has finished executing, but it clearly returned a business/logic error. No retry is needed; throw it directly.
                    log::warn!(
                        "Tool '{}' returned a deterministic error: {}. Aborting execution.",
                        name,
                        e
                    );
                    return Err(e);
                }
                Err(_) => {
                    // A timeout interception occurred
                    if retries == 0 {
                        let reason = if def.is_idempotent {
                            "after max retries"
                        } else {
                            "(non-idempotent tool, no retries allowed)"
                        };
                        return Err(ToolErr(format!(
                            "Tool '{}' execution timed out ({}s) {}",
                            name,
                            timeout_duration.as_secs(),
                            reason
                        )));
                    }

                    retries = retries.saturating_sub(1);
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
