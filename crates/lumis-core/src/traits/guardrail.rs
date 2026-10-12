use crate::error::Result;

/// Aho-Corasick SIMD Guardrail & PII Filter contract.
/// Guarantees O(N + Z) linear-time egress inspection regardless of pattern count.
pub trait GuardrailMatcher: Send + Sync {
    /// Checks if text contains any forbidden or PII patterns.
    fn check_violation(&self, text: &str) -> Option<String>;

    /// Masks all detected PII patterns in-place or returns sanitized string.
    fn mask_pii(&self, text: &str) -> Result<String>;
}
