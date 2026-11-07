#![cfg(feature = "sub-agent")]

use ambi::error::Result;
use ambi::llm::LLMEngineTrait;
use ambi::types::{LLMRequest, Message, Tool, ToolDefinition, ToolErr};
use ambi::{impl_as_any, Agent, AgentState, ChatRunner, LLMEngineConfig, LeafConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::Sender;

#[derive(Deserialize)]
struct AddArgs {
    a: i32,
    b: i32,
}

#[derive(Serialize)]
struct AddOutput {
    sum: i32,
}

struct AddTool;

#[async_trait]
impl Tool for AddTool {
    const NAME: &'static str = "add_numbers";
    type Args = AddArgs;
    type Output = AddOutput;

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Add two integers.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": { "type": "integer" },
                    "b": { "type": "integer" }
                },
                "required": ["a", "b"]
            }),
            timeout_secs: Some(5),
            max_retries: Some(0),
            is_idempotent: true,
        }
    }

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, ToolErr> {
        Ok(AddOutput {
            sum: args.a + args.b,
        })
    }
}

struct MockLeafEngine {
    seen_requests: Arc<Mutex<Vec<LLMRequest>>>,
    in_flight_leaves: Arc<AtomicUsize>,
    max_in_flight_leaves: Arc<AtomicUsize>,
}

#[async_trait]
impl LLMEngineTrait for MockLeafEngine {
    impl_as_any!();

    async fn chat(&self, request: LLMRequest) -> Result<String> {
        self.seen_requests.lock().unwrap().push(request.clone());

        let is_leaf = request.session_id.starts_with("leaf-session-");
        let last_msg = request.history.last().map(|m| (**m).clone());

        if !is_leaf {
            match last_msg {
                Some(Message::User { .. }) => {
                    // Parent turn 1: emit two parallel leaf calls
                    Ok(concat!(
                        "Delegating to two leaves.\n",
                        r#"[TOOL_CALL]{"name":"spawn_leaf","args":{"task":"compute_alpha","context":"ctx_alpha"}}[/TOOL_CALL]"#,
                        "\n",
                        r#"[TOOL_CALL]{"name":"spawn_leaf","args":{"task":"compute_beta"}}[/TOOL_CALL]"#
                    )
                    .to_string())
                }
                Some(Message::Tool { .. }) => {
                    // Parent turn 2: synthesize leaf results
                    let tool_outputs: Vec<String> = request
                        .history
                        .iter()
                        .filter_map(|m| match &**m {
                            Message::Tool { content, .. } => Some(content.clone()),
                            _ => None,
                        })
                        .collect();
                    Ok(format!("Parent combined: {}", tool_outputs.join(" + ")))
                }
                _ => Ok("unexpected parent state".to_string()),
            }
        } else {
            // Inside a leaf call
            let current = self.in_flight_leaves.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_in_flight_leaves
                .fetch_max(current, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(60)).await;
            self.in_flight_leaves.fetch_sub(1, Ordering::SeqCst);

            match last_msg {
                Some(Message::User { content }) => {
                    let text = format!("{:?}", content);
                    if text.contains("compute_alpha") {
                        // Leaf A uses the inherited `add_numbers` tool in round 1
                        Ok(
                            r#"Intermediate leaf thought [TOOL_CALL]{"name":"add_numbers","args":{"a":10,"b":32}}[/TOOL_CALL]"#
                                .to_string(),
                        )
                    } else {
                        Ok("beta_done".to_string())
                    }
                }
                Some(Message::Tool { content, .. }) => Ok(format!("alpha_done({})", content)),
                _ => Ok("unexpected leaf state".to_string()),
            }
        }
    }

    async fn chat_stream(&self, _request: LLMRequest, _tx: Sender<Result<String>>) {}

    fn reset_context(&self) {}
}

#[tokio::test]
async fn test_default_chat_runner_does_not_inject_leaf() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let engine = MockLeafEngine {
        seen_requests: Arc::clone(&seen),
        in_flight_leaves: Arc::new(AtomicUsize::new(0)),
        max_in_flight_leaves: Arc::new(AtomicUsize::new(0)),
    };

    let agent = Agent::make(LLMEngineConfig::Custom(Box::new(engine)))
        .await
        .unwrap()
        .tool(AddTool)
        .unwrap()
        .max_iterations(1);

    let state = AgentState::new_shared("parent-default");
    let runner = ChatRunner::default();
    let _ = runner.chat(&agent, &state, "hello").await;

    let requests = seen.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0].tools.iter().all(|t| t.name != "spawn_leaf"),
        "ChatRunner::default() must not inject spawn_leaf without with_leaf()"
    );
}

#[tokio::test]
async fn test_leaf_parallel_execution_tool_inheritance_and_isolation() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let max_in_flight = Arc::new(AtomicUsize::new(0));
    let engine = MockLeafEngine {
        seen_requests: Arc::clone(&seen),
        in_flight_leaves: Arc::new(AtomicUsize::new(0)),
        max_in_flight_leaves: Arc::clone(&max_in_flight),
    };

    let agent = Agent::make(LLMEngineConfig::Custom(Box::new(engine)))
        .await
        .unwrap()
        .preamble("Parent system prompt")
        .tool(AddTool)
        .unwrap();

    let state = AgentState::new_shared("parent-session");
    let runner = ChatRunner::new(4).with_leaf();

    let response = runner
        .chat(&agent, &state, "Run alpha and beta")
        .await
        .unwrap();

    // Verify both leaf results arrived and intermediate leaf `[TOOL_CALL]` blocks were stripped
    assert!(
        response.contains("alpha_done(") && response.contains("beta_done"),
        "Unexpected parent response: {}",
        response
    );
    assert!(
        !response.contains("Intermediate leaf thought"),
        "Leaf intermediate round text leaked into leaf final answer: {}",
        response
    );

    // Verify both leaves ran in parallel
    assert!(
        max_in_flight.load(Ordering::SeqCst) >= 2,
        "Expected parallel leaf execution, but max in-flight leaves was {}",
        max_in_flight.load(Ordering::SeqCst)
    );

    // Verify structural non-recursion and context isolation on captured requests
    let requests = seen.lock().unwrap();
    let parent_reqs: Vec<&LLMRequest> = requests
        .iter()
        .filter(|r| r.session_id == "parent-session")
        .collect();
    let leaf_reqs: Vec<&LLMRequest> = requests
        .iter()
        .filter(|r| r.session_id.starts_with("leaf-session-"))
        .collect();

    assert_eq!(parent_reqs.len(), 2);
    assert!(
        parent_reqs[0].tools.iter().any(|t| t.name == "spawn_leaf"),
        "Parent request must include spawn_leaf"
    );

    assert!(!leaf_reqs.is_empty());
    for leaf_req in &leaf_reqs {
        assert!(
            leaf_req.tools.iter().all(|t| t.name != "spawn_leaf"),
            "Leaf request must never contain spawn_leaf in tools"
        );
        assert!(
            !leaf_req.tool_prompt.contains("spawn_leaf"),
            "Leaf tool_prompt must never mention spawn_leaf"
        );
        assert!(
            leaf_req.tools.iter().any(|t| t.name == "add_numbers"),
            "Leaf request must inherit parent tools"
        );
        // Leaf history must not contain the parent's user prompt ("Run alpha and beta")
        for msg in &leaf_req.history {
            assert!(
                !msg.to_string().contains("Run alpha and beta"),
                "Leaf history leaked parent conversation"
            );
        }
    }

    // Verify parent state only has User -> Assistant(spawn_leaf) -> Tool -> Tool -> Assistant
    let parent_history_len = state.read().await.chat_history.len();
    assert_eq!(parent_history_len, 5);
}

#[tokio::test]
async fn test_leaf_tool_name_collision_fails_fast() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let engine = MockLeafEngine {
        seen_requests: seen,
        in_flight_leaves: Arc::new(AtomicUsize::new(0)),
        max_in_flight_leaves: Arc::new(AtomicUsize::new(0)),
    };

    let agent = Agent::make(LLMEngineConfig::Custom(Box::new(engine)))
        .await
        .unwrap()
        .tool(AddTool)
        .unwrap();

    let state = AgentState::new_shared("collision-session");
    let runner =
        ChatRunner::default().with_leaf_config(LeafConfig::new().with_tool_name("add_numbers"));

    let err = runner
        .chat(&agent, &state, "should fail")
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("collides with an already registered tool"),
        "Unexpected error: {}",
        err
    );
}

struct LoopingLeafEngine;

#[async_trait]
impl LLMEngineTrait for LoopingLeafEngine {
    impl_as_any!();

    async fn chat(&self, request: LLMRequest) -> Result<String> {
        let is_leaf = request.session_id.starts_with("leaf-session-");
        let last_msg = request.history.last().map(|m| (**m).clone());

        if !is_leaf {
            match last_msg {
                Some(Message::User { .. }) => Ok(
                    r#"[TOOL_CALL]{"name":"spawn_leaf","args":{"task":"loop_forever"}}[/TOOL_CALL]"#
                        .to_string(),
                ),
                Some(Message::Tool { content, .. }) => Ok(format!("Parent got tool result: {}", content)),
                _ => Ok("unexpected".to_string()),
            }
        } else {
            // Leaf keeps calling `add_numbers` on every round to exceed max_iterations
            Ok(r#"[TOOL_CALL]{"name":"add_numbers","args":{"a":1,"b":1}}[/TOOL_CALL]"#.to_string())
        }
    }

    async fn chat_stream(&self, _request: LLMRequest, _tx: Sender<Result<String>>) {}

    fn reset_context(&self) {}
}

#[tokio::test]
async fn test_leaf_max_iterations_mapped_to_tool_error() {
    let agent = Agent::make(LLMEngineConfig::Custom(Box::new(LoopingLeafEngine)))
        .await
        .unwrap()
        .tool(AddTool)
        .unwrap();

    let state = AgentState::new_shared("loop-session");
    let runner = ChatRunner::default().with_leaf_config(LeafConfig::new().with_max_iterations(2));

    let response = runner
        .chat(&agent, &state, "trigger looping leaf")
        .await
        .unwrap();

    assert!(
        response.contains("Leaf sub-agent reached its maximum iteration limit (2)"),
        "Expected structured tool error message in parent response, got: {}",
        response
    );
}
