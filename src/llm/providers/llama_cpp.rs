//! Llama.cpp local inference engine implementation using direct C bindings.
//!
//! This module provides a complete, high-performance local LLM backend backed by
//! the `llama.cpp` library, supporting GPU offloading, KV-cache management,
//! multimodal (vision) inference via MTMD, and entropy evaluation.

pub(crate) mod callback;
pub(crate) mod command;
pub mod config;
pub(crate) mod dispatch;
pub(crate) mod engine;
pub(crate) mod entropy;
pub(crate) mod inference;
pub(crate) mod session;
pub(crate) mod thread;
mod vision;

pub use engine::LlamaEngine;
