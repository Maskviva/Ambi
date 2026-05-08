//! Memory provider trait definitions and in-memory implementations.
//!
//! This module exposes three memory dimensions:
//! - **KV**: exact key-value state for reflexion / settings.
//! - **Semantic**: vector-based long-term recall.
//! - **Summary**: rolling summary for anti-amnesia on context eviction.

pub mod kv;
pub mod semantic;
pub mod summary;

pub use kv::{InMemoryKvProvider, KvMemoryProvider};
pub use semantic::SemanticMemoryProvider;
pub use summary::{InMemorySummaryProvider, SummaryMemoryProvider};
