use aho_corasick::{AhoCorasick, MatchKind};
use lumis_core::error::Result;
use lumis_core::traits::{GuardrailMatcher, GuardrailVerdict};

use super::patterns::{default_credential_patterns, default_injection_patterns};
use super::pii_scanner::PiiScanner;

/// High-performance SIMD Aho-Corasick Guardrail Matcher.
/// Guarantees O(N + Z) linear-time execution across arbitrary number of patterns.
pub struct SimdGuardrailMatcher {
    injection_automaton: AhoCorasick,
    injection_patterns: Vec<String>,
    pii_automaton: AhoCorasick,
    pii_patterns: Vec<String>,
}

impl SimdGuardrailMatcher {
    /// Creates a new matcher using default injection patterns and given PII patterns.
    pub fn new(pii_patterns: Vec<String>) -> Result<Self> {
        Self::with_options(default_injection_patterns(), pii_patterns)
    }

    /// Creates a new matcher with customizable injection and PII patterns.
    pub fn with_options(
        injection_patterns: Vec<String>,
        pii_patterns: Vec<String>,
    ) -> Result<Self> {
        let injection_automaton = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostFirst)
            .build(&injection_patterns)
            .map_err(|e| lumis_core::error::LumisError::Guardrail(e.to_string()))?;

        let pii_automaton = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostFirst)
            .build(&pii_patterns)
            .map_err(|e| lumis_core::error::LumisError::Guardrail(e.to_string()))?;

        Ok(Self {
            injection_automaton,
            injection_patterns,
            pii_automaton,
            pii_patterns,
        })
    }

    #[allow(dead_code)]
    pub fn default_patterns() -> Vec<String> {
        default_credential_patterns()
    }

    fn check_injection(&self, text: &str) -> Option<String> {
        let lower = text.to_lowercase();
        for m in self.injection_automaton.find_iter(&lower) {
            let start_ok = m.start() == 0
                || !lower[..m.start()]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric());
            let end_ok = m.end() == lower.len()
                || !lower[m.end()..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphanumeric());
            if start_ok && end_ok {
                return Some(self.injection_patterns[m.pattern()].clone());
            }
        }
        None
    }
}

impl GuardrailMatcher for SimdGuardrailMatcher {
    fn check_violation(&self, text: &str) -> Option<String> {
        if let Some(inj) = self.check_injection(text) {
            return Some(inj);
        }
        if let Some(m) = self.pii_automaton.find(text) {
            return Some(self.pii_patterns[m.pattern()].clone());
        }
        None
    }

    fn mask_pii(&self, text: &str) -> Result<String> {
        let mut credential_masked = String::with_capacity(text.len());
        self.pii_automaton
            .replace_all_with(text, &mut credential_masked, |_mat, match_text, dst| {
                if match_text.starts_with("sk-") || match_text.starts_with("ghp_") || match_text.starts_with("AKIA") {
                    dst.push_str("[REDACTED_API_KEY]");
                } else {
                    dst.push_str("[REDACTED]");
                }
                true
            });

        let fully_masked = PiiScanner::mask_structured_pii(&credential_masked);
        Ok(fully_masked)
    }

    fn inspect(&self, text: &str) -> GuardrailVerdict {
        if let Some(rule) = self.check_injection(text) {
            return GuardrailVerdict::Blocked {
                reason: "Security guardrail violation: prompt injection or jailbreak heuristic detected".to_string(),
                rule_matched: rule,
            };
        }

        match self.mask_pii(text) {
            Ok(sanitized_text) => GuardrailVerdict::Pass { sanitized_text },
            Err(_) => GuardrailVerdict::Pass {
                sanitized_text: text.to_string(),
            },
        }
    }
}
