use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{Ident, ItemFn, LitStr, Meta, Token, Type};

pub(crate) struct ToolConfig {
    pub(crate) name_override: Option<String>,
    pub(crate) description: String,
    pub(crate) timeout_secs: Option<u64>,
    pub(crate) max_retries: Option<usize>,
    pub(crate) is_idempotent: bool,
    pub(crate) param_descriptions: HashMap<String, String>,
}

pub(crate) struct ArgInfo {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) name_str: String,
    pub(crate) json_type: String,
    pub(crate) is_required: bool,
    pub(crate) description: String,
}

struct ParamDescriptions {
    map: HashMap<String, String>,
}

impl Parse for ParamDescriptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut map = HashMap::new();
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            let value: LitStr = input.parse()?;
            map.insert(key.to_string(), value.value());

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        if !input.is_empty() {
            return Err(input.error("unexpected extra tokens in params(...)"));
        }
        Ok(ParamDescriptions { map })
    }
}

pub(crate) fn parse_tool_config(attr: TokenStream) -> Result<ToolConfig, syn::Error> {
    let mut config = ToolConfig {
        name_override: None,
        description: String::new(),
        timeout_secs: None,
        max_retries: None,
        is_idempotent: false,
        param_descriptions: Default::default(),
    };

    if attr.is_empty() {
        return Ok(config);
    }

    let parser = syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated;
    let parsed = parser.parse(attr)?;

    for meta in parsed {
        match meta {
            Meta::NameValue(nv) => apply_name_value(&nv, &mut config),
            Meta::Path(path) => {
                if path.is_ident("is_idempotent") || path.is_ident("idempotent") {
                    config.is_idempotent = true;
                }
            }
            Meta::List(list) if list.path.is_ident("params") => {
                let parsed_params: ParamDescriptions = list.parse_args()?;
                config.param_descriptions = parsed_params.map;
            }
            _ => {}
        }
    }

    Ok(config)
}

fn apply_name_value(nv: &syn::MetaNameValue, config: &mut ToolConfig) {
    let value = &nv.value;
    if nv.path.is_ident("name") {
        config.name_override = extract_string(value);
    } else if nv.path.is_ident("description") || nv.path.is_ident("desc") {
        if let Some(s) = extract_string(value) {
            config.description = s;
        }
    } else if nv.path.is_ident("timeout_secs") || nv.path.is_ident("timeout") {
        config.timeout_secs = extract_u64(value);
    } else if nv.path.is_ident("max_retries") || nv.path.is_ident("retries") {
        config.max_retries = extract_usize(value);
    } else if nv.path.is_ident("is_idempotent") || nv.path.is_ident("idempotent") {
        if let syn::Expr::Lit(expr_lit) = value {
            if let syn::Lit::Bool(b) = &expr_lit.lit {
                config.is_idempotent = b.value;
            }
        }
    }
}

fn extract_string(expr: &syn::Expr) -> Option<String> {
    if let syn::Expr::Lit(expr_lit) = expr {
        if let syn::Lit::Str(lit) = &expr_lit.lit {
            return Some(lit.value());
        }
    }
    None
}

fn extract_u64(expr: &syn::Expr) -> Option<u64> {
    if let syn::Expr::Lit(expr_lit) = expr {
        if let syn::Lit::Int(lit) = &expr_lit.lit {
            return lit.base10_parse::<u64>().ok();
        }
    }
    None
}

fn extract_usize(expr: &syn::Expr) -> Option<usize> {
    if let syn::Expr::Lit(expr_lit) = expr {
        if let syn::Lit::Int(lit) = &expr_lit.lit {
            return lit.base10_parse::<usize>().ok();
        }
    }
    None
}

pub(crate) fn extract_doc_description(func: &ItemFn) -> String {
    let mut desc = String::new();
    for attr in &func.attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let Meta::NameValue(nv) = &attr.meta {
            if let Some(s) = extract_string(&nv.value) {
                let line = s.trim().to_string();
                if !desc.is_empty() {
                    desc.push('\n');
                }
                desc.push_str(&line);
            }
        }
    }
    desc
}

pub(crate) fn extract_args_info(
    func: &ItemFn,
    param_descriptions: &HashMap<String, String>,
) -> Result<Vec<ArgInfo>, syn::Error> {
    let mut args = Vec::new();
    for input in &func.sig.inputs {
        let pat_type = match input {
            syn::FnArg::Typed(pt) => pt,
            syn::FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    input,
                    "`self` argument is not allowed in tool functions",
                ));
            }
        };
        let ident = match &*pat_type.pat {
            syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            other => {
                return Err(syn::Error::new_spanned(
                    other.clone(),
                    "Tool arguments only support simple identifiers (e.g. `name: Type`)",
                ));
            }
        };
        let ty = (*pat_type.ty).clone();
        let name_str = ident.to_string();
        let (json_type, is_required) = extract_type_info(&ty);
        let description = param_descriptions
            .get(&name_str)
            .cloned()
            .unwrap_or_default();

        args.push(ArgInfo {
            ident,
            ty,
            name_str,
            json_type,
            is_required,
            description,
        });
    }
    Ok(args)
}

pub(crate) fn generate_tool_impl(
    mut func: ItemFn,
    config: ToolConfig,
    description: String,
    args_info: Vec<ArgInfo>,
    output_ty: proc_macro2::TokenStream,
) -> TokenStream {
    let fn_ident = func.sig.ident.clone();
    let struct_name = quote::format_ident!("{}Tool", pascal_case(&fn_ident.to_string()));
    let args_struct_name = quote::format_ident!("{}Args", pascal_case(&fn_ident.to_string()));
    let impl_name = quote::format_ident!("__ambi_tool_impl_{}", fn_ident);
    func.sig.ident = impl_name.clone();

    let tool_name = config.name_override.unwrap_or_else(|| fn_ident.to_string());

    let (arg_idents, arg_types, arg_names_str, arg_json_types, arg_descriptions, required_args) =
        destructure_args(&args_info);

    let timeout_token = config
        .timeout_secs
        .map(|t| quote! { Some(#t) })
        .unwrap_or(quote! { None });
    let retries_token = config
        .max_retries
        .map(|r| quote! { Some(#r) })
        .unwrap_or(quote! { None });
    let is_idempotent = config.is_idempotent;

    let expanded = quote! {
        #func

        #[derive(::serde::Deserialize)]
        pub struct #args_struct_name {
            #( pub #arg_idents: #arg_types, )*
        }

        #[allow(non_camel_case_types)]
        pub struct #struct_name;

        #[async_trait::async_trait]
        impl ::ambi::types::Tool for #struct_name {
            const NAME: &'static str = #tool_name;
            type Args = #args_struct_name;
            type Output = #output_ty;

            fn definition(&self) -> ::ambi::types::ToolDefinition {
                ::ambi::types::ToolDefinition {
                    name: Self::NAME.into(),
                    description: #description.into(),
                    parameters: ::serde_json::json!({
                        "type": "object",
                        "properties": {
                            #(
                                #arg_names_str: {
                                    "type": #arg_json_types,
                                    "description": #arg_descriptions
                                },
                            )*
                        },
                        "required":[ #( #required_args ),* ]
                    }),
                    timeout_secs: #timeout_token,
                    max_retries: #retries_token,
                    is_idempotent: #is_idempotent,
                }
            }

            async fn call(&self, args: Self::Args) -> Result<Self::Output, ::ambi::types::ToolErr> {
                #impl_name(#( args.#arg_idents ),*).await
            }
        }
    };

    TokenStream::from(expanded)
}

#[allow(clippy::type_complexity)]
fn destructure_args(
    args: &[ArgInfo],
) -> (
    Vec<&Ident>,
    Vec<&Type>,
    Vec<&String>, // name_str
    Vec<&String>, // json_type
    Vec<&String>, // description
    Vec<&String>, // required (name)
) {
    let mut idents = Vec::with_capacity(args.len());
    let mut types = Vec::with_capacity(args.len());
    let mut names = Vec::with_capacity(args.len());
    let mut json_types = Vec::with_capacity(args.len());
    let mut descriptions = Vec::with_capacity(args.len());
    let mut required = Vec::with_capacity(args.len());

    for a in args {
        idents.push(&a.ident);
        types.push(&a.ty);
        names.push(&a.name_str);
        json_types.push(&a.json_type);
        descriptions.push(&a.description);
        if a.is_required {
            required.push(&a.name_str);
        }
    }
    (idents, types, names, json_types, descriptions, required)
}

fn pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

pub(crate) fn extract_ok_type(sig: &syn::Signature) -> proc_macro2::TokenStream {
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        if let Type::Path(type_path) = ty.as_ref() {
            if let Some(seg) = type_path.path.segments.last() {
                if seg.ident == "Result" {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(ok)) = args.args.iter().next() {
                            return quote! { #ok };
                        }
                    }
                }
            }
        }
    }
    quote! { () }
}

fn extract_type_info(ty: &Type) -> (String, bool) {
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            let ident = seg.ident.to_string();

            if ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.iter().next() {
                        let (inner_json_type, _) = extract_type_info(inner_ty);
                        return (inner_json_type, false);
                    }
                }
                return ("string".to_string(), false);
            }

            let json_type = match ident.as_str() {
                "String" | "str" | "char" => "string",
                "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
                    "integer"
                }
                "f32" | "f64" => "number",
                "bool" => "boolean",
                "Vec" | "HashSet" | "BTreeSet" | "slice" | "Array" => "array",
                "HashMap" | "BTreeMap" | "Value" => "object",
                _ => "object",
            }
            .to_string();

            return (json_type, true);
        }
    }
    ("string".to_string(), true)
}
