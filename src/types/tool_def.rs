// src/types/tool_def.rs

//! Standard definitions and traits for extending the Agent with custom tools.

use crate::runtime::SendSync;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// A snapshot of a tool's capabilities and constraints, used to inform the LLM.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ToolDefinition {
    /// The unique identifier of the tool (must match the LLM's calling syntax).
    pub name: String,
    /// A clear description instructing the LLM on *when* and *how* to use this tool.
    pub description: String,
    /// JSON schema defining the required arguments.
    pub parameters: Value,

    /// Maximum execution time allowed before the framework aborts the tool.
    #[serde(skip)]
    pub timeout_secs: Option<u64>,

    /// Number of auto-retries permitted if the tool times out (only applies if `is_idempotent` is true).
    #[serde(skip)]
    pub max_retries: Option<usize>,

    /// Marks whether the tool is safe to retry on failure (e.g., fetching data is idempotent; sending an email is not).
    #[serde(skip)]
    pub is_idempotent: bool,
}

/// A generic error wrapper for tool execution failures.
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolErr(pub String);

impl Display for ToolErr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ToolErr {}

/// Defines a custom capability that the Agent can autonomously invoke.
///
/// Implementing `Tool` is the primary way to connect the reasoning engine to external
/// APIs, databases, or local system commands.
///
/// # Cancellation Safety & Timeouts
/// The framework strictly enforces timeouts (default 15s).
/// If your tool modifies external state (e.g., executing a payment or writing to a database),
/// you **MUST** ensure `is_idempotent` is set to `false`. Non-idempotent tools will fail fast
/// on timeout and will **not** be retried, preventing duplicate real-world side effects.
///
/// # Examples
/// ```rust,ignore
/// use ambi::types::{Tool, ToolDefinition, ToolErr};
/// use serde::{Deserialize, Serialize};
/// use async_trait::async_trait;
///
/// #[derive(Deserialize)]
/// struct WeatherArgs { city: String }
///
/// #[derive(Serialize)]
/// struct WeatherResult { temp: f32, condition: String }
///
/// struct WeatherTool;
///
/// #[async_trait]
/// impl Tool for WeatherTool {
///     const NAME: &'static str = "get_weather";
///     type Args = WeatherArgs;
///     type Output = WeatherResult;
///
///     fn definition(&self) -> ToolDefinition {
///         ToolDefinition {
///             name: Self::NAME.to_string(),
///             description: "Get the current weather for a specified city.".to_string(),
///             parameters: serde_json::json!({
///                 "type": "object",
///                 "properties": { "city": { "type": "string" } },
///                 "required": ["city"]
///             }),
///             timeout_secs: Some(10),
///             max_retries: Some(2),
///             is_idempotent: true, // Safe to retry
///         }
///     }
///
///     async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr> {
///         // Simulate fetching weather
///         Ok(WeatherResult { temp: 22.5, condition: "Sunny".to_string() })
///     }
/// }
/// ```
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Tool: SendSync {
    /// The unique name of the tool.
    const NAME: &'static str;

    /// The argument type expected by the tool (must implement `Deserialize`).
    type Args: for<'a> Deserialize<'a>;

    /// The output type returned by the tool (must implement `Serialize`).
    type Output: Serialize;

    /// Retrieves the tool's name dynamically.
    fn name(&self) -> String {
        Self::NAME.to_string()
    }

    /// Provides the framework with the tool's metadata and JSON schema.
    fn definition(&self) -> ToolDefinition;

    /// The actual asynchronous execution block of the tool.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr>;
}

/// An object-safe version of `Tool` used internally by the framework for dynamic dispatch.
/// Users do not need to implement this directly; a blanket implementation is provided.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait DynTool: SendSync {
    /// Retrieves the tool's name.
    fn name(&self) -> String;
    /// Retrieves the tool's definition.
    fn definition(&self) -> ToolDefinition;
    /// Executes the tool using a raw JSON Value as input, returning a JSON Value.
    async fn call_json(&self, args: Value) -> Result<Value, ToolErr>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> DynTool for T
where
    T: Tool + SendSync,
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

/// A generic interface for formatting text streams in real-time.
pub trait StreamFormatter: SendSync {
    /// Pushes a new raw token into the formatter, returning any cleaned, ready-to-display output.
    fn push(&mut self, token: &str) -> String;

    /// Flushes any remaining buffered text at the end of the stream.
    fn flush(&mut self) -> String;
}

/// A generic interface for parsing raw LLM output text into structured tool calls.
pub trait ToolCallParser: SendSync {
    /// Returns the literal start and end tags used by the parser (e.g., `[TOOL_CALL]`, `[/TOOL_CALL]`).
    fn get_tags(&self) -> (String, String);

    /// Formats the JSON tool definitions into a clear system instruction for the LLM.
    fn format_instruction(&self, tools_json: &str) -> String;

    /// Parses a raw string from the LLM, extracting a list of requested tool names and their JSON arguments.
    fn parse(&self, text: &str) -> Vec<(String, Value)>;
}
