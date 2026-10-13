use std::sync::Arc;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use lumis_core::models::scim::{ScimEmail, ScimUser, SCIM_USER_SCHEMA};
use lumis_core::traits::DefaultClaimTranslator;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use tower::ServiceExt;
use uuid::Uuid;

async fn test_app_and_state() -> (axum::Router, AppState) {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Initialize SQLite in-memory driver");
    driver.run_migrations().await.expect("Run migrations");

    let guardrail = Arc::new(
        SimdGuardrailMatcher::new(vec!["sk-ant-".to_string()])
            .expect("Guardrail init"),
    );
    let claim_translator = Arc::new(DefaultClaimTranslator::new());
    let state = AppState::new(
        driver,
        guardrail,
        claim_translator,
        "x-forwarded-user".to_string(),
    );
    let router = create_router(state.clone());
    (router, state)
}

#[tokio::test]
async fn test_federated_multi_idp_identity_linking() {
    let (app, state) = test_app_and_state().await;

    // 1. Initial Local Login creates internal User and local identity
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "pegawai_bps",
            "password": "PasswordBps2026!"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Verify local identity created in database
    let local_idty = state.repo.find_identity("local", "pegawai_bps").await.unwrap();
    assert!(local_idty.is_some());
    let local_rec = local_idty.unwrap();
    assert_eq!(local_rec.provider, "local");
    assert_eq!(local_rec.provider_subject_id, "pegawai_bps");

    let internal_user_id = local_rec.user_id.clone();

    // 2. Link SAML 2.0 (NIP BPS) identity to the same internal user
    let saml_idty = lumis_core::models::UserIdentity {
        id: Uuid::new_v4().to_string(),
        user_id: internal_user_id.clone(),
        provider: "saml".to_string(),
        provider_subject_id: "198801012010121001".to_string(), // NIP BPS
        credential_data: Some(serde_json::json!({ "idp": "rIDM_BPS" })),
        last_login_at: Some(chrono::Utc::now()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    state.repo.link_identity(&saml_idty).await.unwrap();

    // Verify SAML identity resolves to the exact same internal user
    let resolved_saml = state.repo.find_identity("saml", "198801012010121001").await.unwrap();
    assert!(resolved_saml.is_some());
    assert_eq!(resolved_saml.unwrap().user_id, internal_user_id);

    // Verify user has 2 linked identities
    let all_idties = state.repo.list_identities_by_user(&internal_user_id).await.unwrap();
    assert_eq!(all_idties.len(), 2);
}

#[tokio::test]
async fn test_scim_relational_persistence_and_deactivation_cascade() {
    let (_app, state) = test_app_and_state().await;

    // 1. SCIM Provisioning creates scim_users record, internal User, and scim identity
    let scim_user = ScimUser {
        schemas: vec![SCIM_USER_SCHEMA.to_string()],
        id: "usr-scim-999".to_string(),
        user_name: "asn_deputi".to_string(),
        display_name: Some("ASN Deputi Bidang Statistik".to_string()),
        emails: vec![ScimEmail { value: "deputi@bps.go.id".to_string(), primary: true }],
        active: true,
        roles: vec!["member".to_string()],
    };
    let created = state.scim_provisioner.create_user(scim_user).await.unwrap();
    assert_eq!(created.id, "usr-scim-999");

    // Verify internal User exists in relational repository and is active
    let internal_user = state.repo.get_user_by_id("usr-scim-999").await.unwrap();
    assert!(internal_user.is_some());
    let u = internal_user.unwrap();
    assert_eq!(u.username, "asn_deputi");
    assert_eq!(u.is_active, true);

    // Verify federated identity exists for SCIM
    let scim_idty = state.repo.find_identity("scim", "usr-scim-999").await.unwrap();
    assert!(scim_idty.is_some());

    // 2. SCIM Deactivation (PATCH active=false) cascades to internal User.is_active = false
    let deactivated = state.scim_provisioner.update_user_status("usr-scim-999", false).await.unwrap();
    assert!(deactivated);

    let updated_user = state.repo.get_user_by_id("usr-scim-999").await.unwrap().unwrap();
    assert_eq!(updated_user.is_active, false);

    // 3. SCIM Delete cascades to de-provisioning
    let deleted = state.scim_provisioner.delete_user("usr-scim-999").await.unwrap();
    assert!(deleted);
    let after_delete_scim = state.scim_provisioner.get_user("usr-scim-999").await.unwrap();
    assert!(after_delete_scim.is_none());
}
