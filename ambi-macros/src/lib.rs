mod agent_register;
mod tool_register;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

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

#[proc_macro_attribute]
pub fn agent(attr: TokenStream, item: TokenStream) -> TokenStream {
    agent_register::generate_agent_facade(attr, item)
}
