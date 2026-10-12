use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Verdict from guardrail pre-flight inspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardrailVerdict {
    /// Content passes guardrail inspection, containing sanitized text.
    Pass { sanitized_text: String },
    /// Content violates strict security policies (e.g. prompt injection, jailbreak).
    Blocked { reason: String, rule_matched: String },
}

/// Aho-Corasick SIMD Guardrail & PII Filter contract.
/// Guarantees O(N + Z) linear-time egress inspection regardless of pattern count.
pub trait GuardrailMatcher: Send + Sync {
    /// Checks if text contains any forbidden or PII patterns.
    fn check_violation(&self, text: &str) -> Option<String>;

    /// Masks all detected PII patterns in-place or returns sanitized string.
    fn mask_pii(&self, text: &str) -> Result<String>;

    /// Comprehensive pre-flight inspection returning Pass (sanitized) or Blocked verdict.
    fn inspect(&self, text: &str) -> GuardrailVerdict;
}

