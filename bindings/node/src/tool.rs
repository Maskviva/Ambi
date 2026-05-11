use ambi::types::{Tool, ToolDefinition, ToolErr};
use async_trait::async_trait;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use serde_json::Value;

pub struct JsToolBridge {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub timeout_secs: Option<u64>,
    pub max_retries: Option<usize>,
    pub is_idempotent: bool,
    pub callback: ThreadsafeFunction<String, String>,
}

#[async_trait]
impl Tool for JsToolBridge {
    const NAME: &'static str = "JS_DYNAMIC_TOOL";
    type Args = Value;
    type Output = Value;

    fn name(&self) -> String {
        self.name.clone()
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.parameters.clone(),
            timeout_secs: self.timeout_secs,
            max_retries: self.max_retries,
            is_idempotent: self.is_idempotent,
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, ToolErr> {
        let safe_args = if args.is_null() {
            serde_json::json!({})
        } else {
            args
        };
        let args_json = serde_json::to_string(&safe_args)
            .map_err(|e| ToolErr(format!("Serialize args: {}", e)))?;

        let (tx, rx) = tokio::sync::oneshot::channel::<Result<String, napi::Error>>();
        let status = self.callback.call_with_return_value(
            Ok(args_json),
            ThreadsafeFunctionCallMode::Blocking,
            move |res, _env| {
                let _ = tx.send(res);
                Ok(())
            },
        );
        if status != napi::Status::Ok {
            return Err(ToolErr(format!("JS call setup failed: {:?}", status)));
        }

        let result_json = rx
            .await
            .map_err(|_| ToolErr("JS callback channel closed".into()))?
            .map_err(|e| ToolErr(format!("JS callback error: {}", e)))?;

        serde_json::from_str(&result_json)
            .map_err(|e| ToolErr(format!("Invalid JSON result: {}", e)))
    }
}
