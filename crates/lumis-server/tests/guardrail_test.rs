use lumis_core::traits::GuardrailMatcher;

#[path = "../src/guardrail/mod.rs"]
mod guardrail;
use guardrail::SimdGuardrailMatcher;

#[test]
fn test_simd_guardrail_detection() {
    let matcher = SimdGuardrailMatcher::new(vec![
        "sk-".to_string(),
        "password".to_string(),
        "SECRET_KEY".to_string(),
    ])
    .unwrap();

    // Violation detection
    assert_eq!(
        matcher.check_violation("My token is sk-12345abcdef"),
        Some("sk-".to_string())
    );
    assert_eq!(
        matcher.check_violation("The password is admin"),
        Some("password".to_string())
    );
    assert!(matcher.check_violation("Normal safe message here").is_none());

    // PII masking in-place
    let masked = matcher
        .mask_pii("Leaked sk-abcdef with password test")
        .unwrap();
    assert_eq!(masked, "Leaked [REDACTED]abcdef with [REDACTED] test");
}
