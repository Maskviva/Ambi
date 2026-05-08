//! Implementation of the `#[agent]` proc-macro: facade generation, builder pattern, and session management.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{bracketed, parse_macro_input, Ident, ItemStruct, Token, Type};

/// Represents the parsed arguments from the `#[ambi::agent(...)]` attribute.
struct AgentMacroArgs {
    tools: Vec<Type>,
    pipeline: Option<Type>,
}

impl Parse for AgentMacroArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = AgentMacroArgs {
            tools: Vec::new(),
            pipeline: None,
        };

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if key == "tools" {
                let content;
                bracketed!(content in input);
                let parsed_tools =
                    syn::punctuated::Punctuated::<Type, Token![,]>::parse_terminated(&content)?;
                args.tools = parsed_tools.into_iter().collect();
            } else if key == "pipeline" {
                args.pipeline = Some(input.parse()?);
            } else {
                return Err(syn::Error::new_spanned(
                    key,
                    "Unknown argument. Supported arguments are: `tools`, `pipeline`.",
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(args)
    }
}

pub(crate) fn generate_agent_facade(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AgentMacroArgs);
    let target_struct = parse_macro_input!(item as ItemStruct);

    let vis = &target_struct.vis;
    let ident = &target_struct.ident;
    let builder_ident = quote::format_ident!("{}Builder", ident);

    // Determine the pipeline type. Default to the official `::ambi::ChatRunner` if not explicitly specified.
    let is_custom_pipeline = args.pipeline.is_some();
    let pipeline_ty = args
        .pipeline
        .unwrap_or_else(|| syn::parse_quote!(::ambi::ChatRunner));
    let tools = args.tools;

    // Dynamically generate the Builder initialization based on pipeline variation.
    let builder_init = if is_custom_pipeline {
        quote! {
            /// Initializes the Builder with a custom, user-provided execution pipeline.
            pub fn builder(engine: ::ambi::LLMEngineConfig, runner: #pipeline_ty) -> #builder_ident {
                #builder_ident {
                    engine,
                    runner,
                    preamble: String::new(),
                    session_id: None,
                }
            }
        }
    } else {
        quote! {
            /// Initializes the Builder using the framework's default, highly-optimized `ChatRunner`.
            pub fn builder(engine: ::ambi::LLMEngineConfig) -> #builder_ident {
                #builder_ident {
                    engine,
                    runner: ::ambi::ChatRunner::default(),
                    preamble: String::new(),
                    session_id: None,
                }
            }
        }
    };

    let expanded = quote! {
        // --- 1. Facade Entity ---
        /// An auto-generated AI Agent Facade encapsulating the blueprint, runtime state, and execution pipeline.
        #vis struct #ident {
            /// The immutable, high-performance execution blueprint.
            pub agent: ::ambi::Agent,
            /// The runtime context and conversational memory, uniquely isolated for a specific session.
            pub state: ::std::sync::Arc<::tokio::sync::RwLock<::ambi::AgentState>>,
            /// The strictly typed orchestration pipeline ensuring React loops and tool executions.
            runner: #pipeline_ty,
        }

        // --- 2. Dedicated Builder ---
        /// The dedicated builder for constructing and initializing the Agent Facade.
        #vis struct #builder_ident {
            engine: ::ambi::LLMEngineConfig,
            runner: #pipeline_ty,
            preamble: String,
            session_id: Option<String>,
        }

        impl #builder_ident {
            /// Injects the system prompt (persona and instructions) into the agent blueprint.
            /// This intuitively establishes the foundational identity of the Agent.
            pub fn preamble(mut self, text: &str) -> Self {
                self.preamble = text.to_string();
                self
            }

            /// Optionally overrides the deterministic session ID.
            /// If not provided, a globally unique identifier will be dynamically generated during build.
            pub fn session_id(mut self, id: impl Into<String>) -> Self {
                self.session_id = Some(id.into());
                self
            }

            /// Asynchronously constructs the blueprint, initializes the isolated memory state,
            /// and returns the fully operational Agent Facade.
            pub async fn build(self) -> ::ambi::error::Result<#ident> {
                let agent = ::ambi::Agent::make(self.engine).await?
                    .preamble(&self.preamble)
                    #( .tool(#tools)? )*
                    .with_standard_formatting();

                // Core fallback: Struct name + nanosecond timestamp guarantees unique ID generation for macro users.
                let sid = self.session_id.unwrap_or_else(|| {
                    format!("{}_{}", stringify!(#ident), ::std::time::SystemTime::now().duration_since(::std::time::UNIX_EPOCH).unwrap().as_nanos())
                });

                Ok(#ident {
                    agent,
                    state: ::ambi::AgentState::new_shared(sid),
                    runner: self.runner,
                })
            }
        }

        // --- 3. Business Logic Interface ---
        impl #ident {
            #builder_init

            /// Sends a plain text prompt to the agent and awaits the fully synthesized response.
            pub async fn chat(&self, prompt: &str) -> ::ambi::error::Result<String> {
                use ::ambi::agent::pipeline::Pipeline;
                let input = vec![::ambi::ContentPart::Text { text: prompt.to_string() }];
                self.runner.execute(&self.agent, &self.state, input).await
            }

            /// Sends a plain text prompt and returns an asynchronous stream yielding real-time text chunks.
            pub async fn chat_stream(&self, prompt: &str) -> ::ambi::error::Result<::std::pin::Pin<Box<::tokio_stream::wrappers::ReceiverStream<::ambi::error::Result<String>>>>> {
                use ::ambi::agent::pipeline::Pipeline;
                let input = vec![::ambi::ContentPart::Text { text: prompt.to_string() }];
                self.runner.execute_stream(&self.agent, &self.state, input).await
            }

            /// Executes a sophisticated multi-modal request (e.g., intertwining plain text with base64 images).
            pub async fn execute(&self, input: Vec<::ambi::ContentPart>) -> ::ambi::error::Result<String> {
                use ::ambi::agent::pipeline::Pipeline;
                self.runner.execute(&self.agent, &self.state, input).await
            }

            /// Executes a multi-modal request returning a real-time token stream.
            pub async fn execute_stream(&self, input: Vec<::ambi::ContentPart>) -> ::ambi::error::Result<::std::pin::Pin<Box<::tokio_stream::wrappers::ReceiverStream<::ambi::error::Result<String>>>>> {
                use ::ambi::agent::pipeline::Pipeline;
                self.runner.execute_stream(&self.agent, &self.state, input).await
            }

            /// Directly injects volatile background context (e.g., Retrieval-Augmented Generation results).
            /// Safely bypasses conversational eviction and seamlessly influences the LLM's next inference step.
            pub async fn set_dynamic_context(&self, context: &str) {
                self.state.write().await.set_dynamic_context(context);
            }

            /// Appends context to the existing dynamic background knowledge.
            pub async fn append_dynamic_context(&self, context: &str) {
                self.state.write().await.append_dynamic_context(context);
            }

            /// Flushes and wipes the dynamic context.
            pub async fn clear_dynamic_context(&self) {
                self.state.write().await.clear_dynamic_context();
            }

            /// Empties the short-term working memory (chat history) and hard-resets the underlying Key-Value Cache.
            pub async fn clear_history(&self) {
                let mut lock = self.state.write().await;
                lock.clear_history(&self.agent);
            }
        }
    };

    TokenStream::from(expanded)
}
