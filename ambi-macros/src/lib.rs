//! Procedural macros for the Ambi AI agent framework.
//!
//! Provides two attribute macros:
//! - `#[tool]` — wraps an async function into an `ambi::types::Tool` implementor.
//! - `#[agent]` — generates a facade struct with a builder and convenience methods.

mod agent_register;
mod tool_register;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

/// Marks an async function as an autonomous tool the agent can invoke.
///
/// # Attributes
/// - `name` — optional override for the tool name (defaults to function name).
/// - `description` / `desc` — optional description (extracted from doc comments by default).
/// - `timeout` / `timeout_secs` — max execution time in seconds.
/// - `max_retries` / `retries` — number of auto-retries on timeout (idempotent only).
/// - `idempotent` / `is_idempotent` — marks the tool as safe to retry.
/// - `params(key = "description", ...)` — per-parameter descriptions.
#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let config = match tool_register::parse_tool_config(attr) {
        Ok(c) => c,
        Err(e) => return e.to_compile_error().into(),
    };

    let description = if config.description.is_empty() {
        tool_register::extract_doc_description(&func)
    } else {
        config.description.clone()
    };

    let args_info = match tool_register::extract_args_info(&func, &config.param_descriptions) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    let output_ty = tool_register::extract_ok_type(&func.sig);

    tool_register::generate_tool_impl(func, config, description, args_info, output_ty)
}

/// Generates an agent facade with a builder, session management, and convenience methods.
///
/// # Attributes
/// - `tools = [...]` — list of tool types to register.
/// - `pipeline = ...` — optional custom pipeline type (defaults to `ChatRunner`).
#[proc_macro_attribute]
pub fn agent(attr: TokenStream, item: TokenStream) -> TokenStream {
    agent_register::generate_agent_facade(attr, item)
}
