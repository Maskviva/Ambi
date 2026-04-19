use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolErr(pub String);

impl Display for ToolErr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ToolErr {}

#[async_trait]
pub trait Tool: Send + Sync {
    const NAME: &'static str;

    type Args: for<'a> Deserialize<'a>;
    type Output: Serialize;

    fn name(&self) -> String {
        Self::NAME.to_string()
    }

    fn definition(&self) -> ToolDefinition;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr>;
}

#[async_trait]
pub trait DynTool: Send + Sync {
    fn name(&self) -> String;

    fn definition(&self) -> ToolDefinition;

    async fn call_json(&self, args: Value) -> Result<Value, ToolErr>;
}

#[async_trait]
impl<T> DynTool for T
where
    T: Tool + Send + Sync,
{
    fn name(&self) -> String {
        Tool::name(self)
    }

    fn definition(&self) -> ToolDefinition {
        Tool::definition(self)
    }

    async fn call_json(&self, args: Value) -> Result<Value, ToolErr> {
        let parsed: T::Args = serde_json::from_value(args).map_err(|e| ToolErr(e.to_string()))?;

        let result = Tool::call(self, parsed).await?;

        serde_json::to_value(result).map_err(|e| ToolErr(e.to_string()))
    }
}

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
            if let Some(end) = text.find("[/TOOL_CALL]") {
                let json_part = &text[start + 11..end];
                if let Ok(val) = serde_json::from_str::<Value>(json_part.trim()) {
                    if let (Some(name), Some(args)) =
                        (val.get("name").and_then(|n| n.as_str()), val.get("args"))
                    {
                        return Some((name.to_string(), args.clone()));
                    }
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
        match tool_map.get(&name) {
            Some(tool) => {
                let result = tool.call_json(args.clone()).await?;

                Ok(serde_json::to_string(&result).map_err(|e| ToolErr(e.to_string()))?)
            }

            None => Err(ToolErr(format!("Tool {} not found", name))),
        }
    }
}
