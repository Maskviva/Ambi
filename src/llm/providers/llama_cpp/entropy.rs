// src/llm/providers/llama_cpp/engine/entropy.rs

use super::session::InferenceSession;
use crate::error::{AmbiError, Result};
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::{AddBos, LlamaModel};

impl InferenceSession {
    /// Evaluate the average per‑token entropy of `sentence`.
    ///
    /// Tokenizes the input, evaluates all tokens in one batch, and computes the
    /// entropy for each token position using the logits. Returns the arithmetic
    /// mean across tokens.
    ///
    /// # Errors
    /// Returns `EngineError` if tokenization, batch building, or decoding fails.
    pub(crate) fn evaluate_entropy(
        sentence: &str,
        model: &LlamaModel,
        context: &mut LlamaContext,
        mut batch: &mut LlamaBatch,
        session: &mut InferenceSession,
    ) -> Result<f32> {
        // 1. Tokenize
        let tokens = model
            .str_to_token(sentence, AddBos::Always)
            .map_err(|e| AmbiError::EngineError(format!("Tokenize failed: {}", e)))?
            .to_vec();

        if tokens.is_empty() {
            return Ok(0.0);
        }

        // 2. Build batch with consecutive positions.
        batch.clear();
        for (i, &t) in tokens.iter().enumerate() {
            batch
                .add(t, session.pos + i as i32, &[0], true)
                .map_err(|e| AmbiError::EngineError(format!("Batch add failed: {}", e)))?;
        }

        // 3. Run forward pass to populate logits.
        context
            .decode(&mut batch)
            .map_err(|e| AmbiError::EngineError(format!("Decoding failed: {}", e)))?;

        // 4. Accumulate entropy over all token positions.
        let mut total_entropy = 0.0_f32;
        for i in 0..tokens.len() {
            let logits = context.get_logits_ith(i as i32);
            total_entropy += Self::token_entropy(logits);
        }

        session.pos += tokens.len() as i32;

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
