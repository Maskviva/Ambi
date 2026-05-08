//! Token-aware context packing for RAG pipelines.
//!
//! Sorts documents by descending score and packs them into a single context
//! string while respecting a user-defined token budget.

use super::document::Document;
use ambi::error::Result;
use ambi::llm::LLMEngine;
use std::sync::Arc;

/// Token-aware packer that fits the highest-scoring documents into the context window.
pub struct ContextPacker;

impl ContextPacker {
    /// Packs the given documents into a formatted context string.
    ///
    /// Documents are sorted by score descending; packing stops when adding the
    /// next document would exceed `max_context_tokens`.
    pub fn pack(
        engine: &Arc<LLMEngine>,
        docs: Vec<Document>,
        max_context_tokens: usize,
    ) -> Result<String> {
        let mut sorted_docs = docs;
        sorted_docs.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut packed = String::from("[BACKGROUND KNOWLEDGE]:\n");
        let prefix_tokens = engine.count_tokens(&packed)?;
        let mut current_tokens = prefix_tokens;
        let mut accepted_count = 0;

        for doc in sorted_docs {
            let chunk = format!(
                "-[Citation: {}] (Score: {:.2}): {}\n",
                doc.id, doc.score, doc.content
            );
            let chunk_tokens = engine.count_tokens(&chunk)?;

            if current_tokens + chunk_tokens > max_context_tokens {
                break;
            }

            packed.push_str(&chunk);
            current_tokens += chunk_tokens;
            accepted_count += 1;
        }

        if accepted_count == 0 {
            return Ok(String::new());
        }

        packed.push_str(
            "\n[INSTRUCTION]: Please answer the user's question primarily based on the BACKGROUND KNOWLEDGE above. If the information is insufficient, state it clearly. Always refer to the Citation ID if possible.\n",
        );

        Ok(packed)
    }
}
