//! Trait definitions for answer extraction and aggregation in Self-Consistency pipelines.

use std::collections::HashMap;

/// Extracts a canonical answer string from a raw LLM response.
pub trait AnswerExtractor: Send + Sync {
    fn extract(&self, raw_response: &str) -> Option<String>;
}

/// Aggregates multiple extracted answers into a single consensus result.
pub trait Aggregator: Send + Sync {
    fn aggregate(&self, extracted_answers: &[Option<String>]) -> Option<(usize, String)>;
}

/// Extracts text delimited by a known prefix/suffix pair.
pub struct PatternExtractor {
    pub prefix: String,
    pub suffix: String,
}

impl PatternExtractor {
    pub fn new(prefix: &str, suffix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
        }
    }
}

impl AnswerExtractor for PatternExtractor {
    fn extract(&self, raw: &str) -> Option<String> {
        if let Some(start_idx) = raw.find(&self.prefix) {
            let content_start = start_idx + self.prefix.len();
            if let Some(end_idx) = raw[content_start..].find(&self.suffix) {
                return Some(
                    raw[content_start..content_start + end_idx]
                        .trim()
                        .to_string(),
                );
            }
            return Some(raw[content_start..].trim().to_string());
        }
        None
    }
}

/// Aggregates answers by simple majority (plurality) voting.
pub struct MajorityVoting;

impl Aggregator for MajorityVoting {
    fn aggregate(&self, extracted_answers: &[Option<String>]) -> Option<(usize, String)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut first_seen: HashMap<String, usize> = HashMap::new();

        for (i, ans) in extracted_answers.iter().enumerate() {
            if let Some(val) = ans {
                *counts.entry(val.clone()).or_insert(0) += 1;
                first_seen.entry(val.clone()).or_insert(i);
            }
        }

        let mut max_count = 0;
        let mut best_ans = None;

        for (val, count) in counts {
            if count > max_count {
                max_count = count;
                best_ans = Some(val);
            }
        }

        best_ans.map(|ans| (*first_seen.get(&ans).unwrap(), ans))
    }
}
