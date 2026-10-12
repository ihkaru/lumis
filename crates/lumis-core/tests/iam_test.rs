use std::collections::HashMap;
use lumis_core::models::EnterpriseRole;
use lumis_core::traits::{ClaimTranslator, DefaultClaimTranslator};


#[test]
fn test_saml_ad_header_translation() {
    let translator = DefaultClaimTranslator::new();
    let mut headers = HashMap::new();
    headers.insert("x-auth-request-user".to_string(), "199201012015021001".to_string());
    headers.insert("x-auth-request-email".to_string(), "budi@agency.go.id".to_string());
    headers.insert(
        "x-auth-request-groups".to_string(),
        "pegawai, data_analyst, operator".to_string(),
    );
    headers.insert("x-department".to_string(), "Direktorat Metodologi".to_string());
    headers.insert("x-nip".to_string(), "199201012015021001".to_string());

    let claims = translator.extract_claims(&headers).expect("Should extract claims");
    assert_eq!(claims.username, "199201012015021001");
    assert_eq!(claims.email.as_deref(), Some("budi@agency.go.id"));
    assert_eq!(claims.roles.len(), 3);
    assert_eq!(claims.department.as_deref(), Some("Direktorat Metodologi"));
    assert_eq!(claims.employee_id.as_deref(), Some("199201012015021001"));

    // Multi-group highest monotonic role precedence
    let effective_role = translator.resolve_effective_role(&claims);
    assert_eq!(effective_role, EnterpriseRole::PowerUser);
    assert_eq!(effective_role.level(), 3);
}

#[test]
fn test_legacy_aliases_and_ciso_precedence() {
    let translator = DefaultClaimTranslator::new();
    let mut headers = HashMap::new();
    headers.insert("x-forwarded-user".to_string(), "ciso_officer".to_string());
    headers.insert(
        "x-forwarded-groups".to_string(),
        "viewer; pegawai; ciso".to_string(),
    );

    let claims = translator.extract_claims(&headers).expect("Should extract claims");
    let effective = translator.resolve_effective_role(&claims);
    assert_eq!(effective, EnterpriseRole::SuperAdmin);
    assert_eq!(effective.level(), 5);
}

#[test]
fn test_model_clearance_enforcement_matrix() {
    let translator = DefaultClaimTranslator::new();

    // Guest (Level 1) can only access TierBasic
    assert!(translator.evaluate_model_clearance(EnterpriseRole::Guest, "gpt-4o-mini").is_ok());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::Guest, "gemini-1.5-flash").is_ok());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::Guest, "gpt-4o").is_err());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::Guest, "deepseek-r1").is_err());

    // Member (Level 2) can access TierBasic and TierStandard
    assert!(translator.evaluate_model_clearance(EnterpriseRole::Member, "gpt-4o-mini").is_ok());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::Member, "gpt-4o").is_ok());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::Member, "gemini-1.5-pro").is_ok());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::Member, "deepseek-coder").is_err());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::Member, "deepseek-r1").is_err());

    // PowerUser (Level 3) can access all tiers including TierAdvanced and TierReasoning
    assert!(translator.evaluate_model_clearance(EnterpriseRole::PowerUser, "deepseek-coder").is_ok());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::PowerUser, "claude-3-5-sonnet").is_ok());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::PowerUser, "deepseek-r1").is_ok());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::PowerUser, "o1-mini").is_ok());

    // SuperAdmin (Level 5) can access everything
    assert!(translator.evaluate_model_clearance(EnterpriseRole::SuperAdmin, "deepseek-r1").is_ok());
    assert!(translator.evaluate_model_clearance(EnterpriseRole::SuperAdmin, "o1").is_ok());
}

#[test]
fn test_unauthorized_when_missing_identity() {
    let translator = DefaultClaimTranslator::new();
    let headers = HashMap::new();
    assert!(translator.extract_claims(&headers).is_err());
}
