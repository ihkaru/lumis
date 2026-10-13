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

#[tokio::test]
async fn test_scim_rfc7644_and_inactive_account_pep_rejection() {
    let (app, state) = test_app_and_state().await;

    // 1. Create user via SCIM without specifying `id` (RFC 7644 server-generated ID)
    let scim_create_req = Request::builder()
        .method("POST")
        .uri("/scim/v2/Users")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
            "userName": "rudi.hartono@bps.go.id",
            "displayName": "Rudi Hartono",
            "emails": [{"value": "rudi.hartono@bps.go.id", "primary": true}],
            "active": true
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(scim_create_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let scim_user_resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let generated_id = scim_user_resp["id"].as_str().unwrap().to_string();
    assert!(!generated_id.is_empty());

    // 2. Set password for rudi so he can do local login
    let rudi_user = state.repo.get_user_by_username("rudi.hartono@bps.go.id").await.unwrap().unwrap();
    let mut rudi_with_pass = rudi_user.clone();
    rudi_with_pass.password_hash = Some(lumis_core::models::User::hash_password("RudiPass123!"));
    state.repo.upsert_user(&rudi_with_pass).await.unwrap();

    // 3. Normal login succeeds
    let login_req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "rudi.hartono@bps.go.id",
            "password": "RudiPass123!"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(login_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 4. Deactivate Rudi using RFC 7644 SCIM PATCH Operations
    let patch_req = Request::builder()
        .method("PATCH")
        .uri(format!("/scim/v2/Users/{}", generated_id))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "schemas": ["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
            "Operations": [
                {
                    "op": "replace",
                    "path": "active",
                    "value": false
                }
            ]
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(patch_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 5. Verification: Login is rejected with 403 Forbidden
    let login_blocked_req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "rudi.hartono@bps.go.id",
            "password": "RudiPass123!"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(login_blocked_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 6. Verification: auth/me with cookie is rejected with 403 Forbidden
    let me_req = Request::builder()
        .method("GET")
        .uri("/api/auth/me")
        .header(header::COOKIE, "lumis_user=rudi.hartono@bps.go.id")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(me_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 7. Verification: creating session is rejected with 403 Forbidden
    let session_req = Request::builder()
        .method("POST")
        .uri("/api/sessions")
        .header(header::COOKIE, "lumis_user=rudi.hartono@bps.go.id")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "title": "Blocked Session",
            "model_id": "gemini-2.5-flash"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(session_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 8. Bootstrap superadmin to test re-activation
    let admin_user = lumis_core::models::User {
        id: "admin-id".to_string(),
        username: "admin".to_string(),
        email: None,
        role: lumis_core::models::EnterpriseRole::SuperAdmin,
        tenant_id: None,
        preferences: serde_json::json!({}),
        is_active: true,
        password_hash: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    state.repo.upsert_user(&admin_user).await.unwrap();

    // 9. Admin reactivates Rudi via /api/admin/users/status
    let reactivate_req = Request::builder()
        .method("POST")
        .uri("/api/admin/users/status")
        .header(header::COOKIE, "lumis_user=admin")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "rudi.hartono@bps.go.id",
            "is_active": true
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(reactivate_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 10. Login succeeds again!
    let login_retry = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "rudi.hartono@bps.go.id",
            "password": "RudiPass123!"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(login_retry).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
