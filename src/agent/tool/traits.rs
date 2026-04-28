use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,

    #[serde(skip)]
    pub timeout_secs: Option<u64>,

    #[serde(skip)]
    pub max_retries: Option<usize>,

    #[serde(skip)]
    pub is_idempotent: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolErr(pub String);

impl Display for ToolErr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ToolErr {}

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

/// # Cancellation Safety
///
/// The framework will automatically apply a timeout control to `Tool` (using `tokio::time::timeout`).
/// This means the Future returned by the `call` method may be dropped at any `.await` point.
///
/// 1. If your Tool is non-idempotent (such as payment or placing an order),
///    please ensure that your internal logic is **cancel-safe**.
///    It is recommended that non-idempotent tools internally manage timeout logic in a self-contained manner,
///    set the tool's defined `is_idempotent` to `false`, and set `timeout_secs` to a sufficiently
///    long duration to prevent being prematurely cut off by the framework.
///
/// 2. Avoid leaking `tokio::task::spawn_blocking` in tools, unless you know how to safely stop background threads.
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

pub trait StreamFormatter: Send + Sync {
    fn push(&mut self, token: &str) -> String;
    fn flush(&mut self) -> String;
}

pub trait ToolCallParser: Send + Sync {
    fn get_tags(&self) -> (String, String);
    fn format_instruction(&self, tools_json: &str) -> String;
    fn parse(&self, text: &str) -> Vec<(String, Value)>;

    fn create_stream_formatter(&self) -> Box<dyn StreamFormatter>;
}
