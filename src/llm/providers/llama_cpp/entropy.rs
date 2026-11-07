// src/llm/providers/llama_cpp/entropy.rs

use super::session::InferenceSession;
use crate::error::{AmbiError, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::LlamaModel;

impl InferenceSession {
    /// Evaluate the average per-token entropy of `sentence` without touching the chat context.
    ///
    /// The sentence is decoded in a context created for this call. That keeps the measurement from
    /// advancing the chat context's decode position, which a range clear of the KV cache does not
    /// rewind.
    pub(crate) fn evaluate_entropy(
        sentence: &str,
        model: &LlamaModel,
        backend: &LlamaBackend,
        n_batch: u32,
    ) -> Result<f32> {
        let tokens = model.vocab().tokenize(sentence.as_bytes(), true, true);

        if tokens.is_empty() {
            return Ok(0.0);
        }

        // Keep a sane floor: a context sized to a handful of tokens is not worth creating, and
        // some backends reject very small contexts outright.
        let n_ctx = u32::try_from(tokens.len())
            .map_err(|_| {
                AmbiError::engine_kind(
                    crate::error::EngineErrorKind::InvalidRequest,
                    "Entropy input exceeds context limits".to_string(),
                )
            })?
            .max(64);
        let mut params = LlamaContextParams::default().with_n_ctx(Some(
            std::num::NonZeroU32::new(n_ctx).expect("n_ctx is floored above zero"),
        ));
        params = params.with_n_batch(n_batch.max(1));

        let mut context = model
            .new_context(backend, params)
            .map_err(|e| AmbiError::engine(format!("Failed to create entropy context: {}", e)))?;
        let mut batch = LlamaBatch::new(tokens.len().max(1), 1);
        for (i, &token) in tokens.iter().enumerate() {
            batch
                .add(token, i as i32, &[0], true)
                .map_err(|e| AmbiError::engine(format!("Batch add failed: {}", e)))?;
        }
        context
            .decode(&mut batch)
            .map_err(|e| AmbiError::engine(format!("Decoding failed: {}", e)))?;

        let total_entropy: f32 = (0..tokens.len())
            .map(|i| Self::token_entropy(context.get_logits_ith(i as i32)))
            .sum();
        Ok(total_entropy / tokens.len() as f32)
    }

    /// Numerically stable entropy of a single token distribution from logits.
    ///
    /// Uses two passes over `logits` to avoid heap allocations while preventing
    /// overflow via the max‑logit subtraction.
    fn token_entropy(logits: &[f32]) -> f32 {
        let max_logit = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        // First pass: compute the normalising constant.
        let sum_exp: f32 = logits.iter().map(|&l| (l - max_logit).exp()).sum();

        // Second pass: compute entropy, ignoring probabilities ≤ 1e‑7 to avoid ln(0).
        logits
            .iter()
            .map(|&l| {
                let p = (l - max_logit).exp() / sum_exp;
                if p > 1e-7 {
                    -p * p.ln()
                } else {
                    0.0
                }
            })
            .sum()
    }
}
