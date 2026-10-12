use aho_corasick::{AhoCorasick, MatchKind};
use lumis_core::error::Result;
use lumis_core::traits::GuardrailMatcher;

/// Production-Grade SIMD Aho-Corasick Guardrail Matcher.
/// Guarantees O(N + Z) linear-time execution across arbitrary number of patterns.
pub struct SimdGuardrailMatcher {
    automaton: AhoCorasick,
    patterns: Vec<String>,
}

impl SimdGuardrailMatcher {
    pub fn new(patterns: Vec<String>) -> Result<Self> {
        let automaton = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostFirst)
            .build(&patterns)
            .map_err(|e| lumis_core::error::LumisError::Guardrail(e.to_string()))?;

        Ok(Self { automaton, patterns })
    }

    #[allow(dead_code)]
    pub fn default_patterns() -> Vec<String> {
        vec![
            "sk-".to_string(),
            "ghp_".to_string(),
            "password".to_string(),
            "secret_key".to_string(),
        ]
    }
}

impl GuardrailMatcher for SimdGuardrailMatcher {
    fn check_violation(&self, text: &str) -> Option<String> {
        if let Some(m) = self.automaton.find(text) {
            let matched_pattern = &self.patterns[m.pattern()];
            Some(matched_pattern.clone())
        } else {
            None
        }
    }

    fn mask_pii(&self, text: &str) -> Result<String> {
        let mut result = String::with_capacity(text.len());
        self.automaton
            .replace_all_with(text, &mut result, |_mat, _match_text, dst| {
                dst.push_str("[REDACTED]");
                true
            });
        Ok(result)
    }
}
