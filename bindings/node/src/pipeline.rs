// bindings/node/src/pipeline.rs

use super::agent::JsAgent;
use super::agent::JsAgentState;
use super::types::JsContentPart;
use ambi::agent::pipeline::chat_runner::ChatRunner;
use ambi::agent::pipeline::Pipeline;
use ambi::ContentPart;
use napi::bindgen_prelude::Function;
use napi::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunctionCallMode};
use napi_derive::napi;

// ── Helpers ──

fn mk_tsfn_s(
    f: &Function<'_, String, ()>,
) -> napi::Result<
    napi::threadsafe_function::ThreadsafeFunction<
        String,
        (),
        String,
        napi::Status,
        false,
        false,
        0,
    >,
> {
    f.build_threadsafe_function::<String>()
        .build_callback(|ctx: ThreadsafeCallContext<String>| Ok(ctx.value))
}

fn mk_tsfn_unit(
    f: &Function<'_, (), ()>,
) -> napi::Result<
    napi::threadsafe_function::ThreadsafeFunction<(), (), (), napi::Status, false, false, 0>,
> {
    f.build_threadsafe_function::<()>()
        .build_callback(|_ctx: ThreadsafeCallContext<()>| Ok(()))
}

// ── ChatRunner ──

/// The default ReAct execution loop. Call `chat` for a simple text
/// prompt, or `execute` to pass raw multimodal content parts.
/// Streaming variants deliver tokens through callbacks.
#[napi]
pub struct JsChatRunner;

#[napi]
impl JsChatRunner {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self
    }

    // ── Sync Chat ──

    /// Single-turn text chat. High-level shorthand for `execute`.
    #[napi]
    pub async fn chat(
        &self,
        agent: &JsAgent,
        state: &JsAgentState,
        prompt: String,
    ) -> napi::Result<String> {
        ChatRunner::default()
            .chat(&agent.inner, &state.inner, &prompt)
            .await
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    // ── Streaming Chat ──

    /// Stream the agent's response token by token. Fires `onComplete`
    /// when the loop finishes, or `onError` if something goes wrong.
    ///
    /// ```js
    /// runner.chatStream(
    ///   agent, state, "Tell me a story",
    ///   (token) => process.stdout.write(token),
    ///   ()      => console.log("done"),
    ///   (err)   => console.error(err),
    /// );
    /// ```
    #[napi]
    pub fn chat_stream(
        &self,
        agent: &JsAgent,
        state: &JsAgentState,
        prompt: String,
        on_token: Function<'_, String, ()>,
        on_complete: Function<'_, (), ()>,
        on_error: Function<'_, String, ()>,
    ) -> napi::Result<()> {
        let tsfn_token = mk_tsfn_s(&on_token)?;
        let tsfn_complete = mk_tsfn_unit(&on_complete)?;
        let tsfn_error = mk_tsfn_s(&on_error)?;

        let agent_clone = agent.inner.clone();
        let state_clone = state.inner.clone();

        tokio::spawn(async move {
            let stream = match ChatRunner::default()
                .chat_stream(&agent_clone, &state_clone, &prompt)
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    tsfn_error.call(e.to_string(), ThreadsafeFunctionCallMode::NonBlocking);
                    return;
                }
            };

            use tokio_stream::StreamExt;
            let mut stream = stream;

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        tsfn_token.call(text, ThreadsafeFunctionCallMode::NonBlocking);
                    }
                    Err(e) => {
                        tsfn_error.call(e.to_string(), ThreadsafeFunctionCallMode::NonBlocking);
                        return;
                    }
                }
            }

            tsfn_complete.call((), ThreadsafeFunctionCallMode::NonBlocking);
        });

        Ok(())
    }

    // ── Low-level Execute ──

    /// Execute the full pipeline with raw multimodal input.
    #[napi]
    pub async fn execute(
        &self,
        agent: &JsAgent,
        state: &JsAgentState,
        input: Vec<JsContentPart>,
    ) -> napi::Result<String> {
        let parts: Vec<ContentPart> = input.into_iter().map(|p| p.into()).collect();
        ChatRunner::default()
            .execute(&agent.inner, &state.inner, parts)
            .await
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Streaming variant of `execute`.
    #[napi]
    pub fn execute_stream(
        &self,
        agent: &JsAgent,
        state: &JsAgentState,
        input: Vec<JsContentPart>,
        on_token: Function<'_, String, ()>,
        on_complete: Function<'_, (), ()>,
        on_error: Function<'_, String, ()>,
    ) -> napi::Result<()> {
        let tsfn_token = mk_tsfn_s(&on_token)?;
        let tsfn_complete = mk_tsfn_unit(&on_complete)?;
        let tsfn_error = mk_tsfn_s(&on_error)?;

        let parts: Vec<ContentPart> = input.into_iter().map(|p| p.into()).collect();
        let agent_clone = agent.inner.clone();
        let state_clone = state.inner.clone();

        tokio::spawn(async move {
            let stream = match ChatRunner::default()
                .execute_stream(&agent_clone, &state_clone, parts)
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    tsfn_error.call(e.to_string(), ThreadsafeFunctionCallMode::NonBlocking);
                    return;
                }
            };

            use tokio_stream::StreamExt;
            let mut stream = stream;

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        tsfn_token.call(text, ThreadsafeFunctionCallMode::NonBlocking);
                    }
                    Err(e) => {
                        tsfn_error.call(e.to_string(), ThreadsafeFunctionCallMode::NonBlocking);
                        return;
                    }
                }
            }

            tsfn_complete.call((), ThreadsafeFunctionCallMode::NonBlocking);
        });

        Ok(())
    }

    // ── Lifecycle ──

    /// Nuke both the conversation history and the engine's KV cache.
    #[napi]
    pub async fn clear_history(agent: &JsAgent, state: &JsAgentState) -> napi::Result<()> {
        let mut state_guard = state.inner.write().await;
        ChatRunner::clear_history(&agent.inner, &mut *state_guard);
        Ok(())
    }
}
