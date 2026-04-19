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

pub mod core;

pub use crate::core::agent::Agent;
pub use crate::core::llm::{EngineBackend, EngineConfig};
pub use crate::core::tool::{Tool, ToolDefinition};

#[cfg(feature = "local")]
pub use crate::core::llm::LlamaEngineConfig;

#[cfg(feature = "cloud")]
pub use crate::core::llm::OpenAIEngineConfig;
