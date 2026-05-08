// ambi-memory/src/lib.rs

//! Pluggable, Multi-dimensional Cognitive Memory System for the Ambi AI framework.

pub mod error;
pub mod extension;
pub mod provider;

pub use error::{MemoryError, Result};
pub use extension::AgentStateMemoryExt;

// Export all providers
pub use provider::kv::{InMemoryKvProvider, KvMemoryProvider};
pub use provider::semantic::SemanticMemoryProvider;
pub use provider::summary::{InMemorySummaryProvider, SummaryMemoryProvider};
