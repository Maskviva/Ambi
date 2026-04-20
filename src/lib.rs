#[cfg(any(
    all(feature = "cuda", feature = "vulkan"),
    all(feature = "cuda", feature = "metal"),
    all(feature = "cuda", feature = "rocm"),
    all(feature = "vulkan", feature = "metal"),
    all(feature = "vulkan", feature = "rocm"),
    all(feature = "metal", feature = "rocm")
))]
compile_error!(
    "Cannot enable multiple LLM hardware acceleration backends. Please select only one of: cuda, vulkan, metal, rocm."
);

pub mod agent;
pub mod llm;

pub use crate::agent::Agent;
pub use crate::agent::{Tool, ToolDefinition};
pub use crate::llm::{EngineBackend, EngineConfig};

#[cfg(feature = "local")]
pub use crate::llm::LlamaEngineConfig;

#[cfg(feature = "cloud")]
pub use crate::llm::OpenAIEngineConfig;
