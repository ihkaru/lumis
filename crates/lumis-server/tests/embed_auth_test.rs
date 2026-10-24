use std::sync::Arc;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use lumis_core::traits::DefaultClaimTranslator;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use serde_json::Value;
use tower::ServiceExt;

async fn test_app() -> axum::Router {
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
    create_router(state)
}

#[tokio::test]
async fn test_embed_token_issuance_verification_and_auth_roundtrip() {
    let app = test_app().await;

    // 1. Issue an embed token for external host user
    let issue_req = Request::builder()
        .method("POST")
        .uri("/api/auth/embed-token")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "external_user_id": "host_usr_8812",
            "username": "sarah.connor",
            "tenant_id": "corp_sp",
            "roles": ["member"],
            "email": "sarah@host.corp",
            "ttl_seconds": 1800
        }).to_string()))
        .unwrap();

    let issue_res = app.clone().oneshot(issue_req).await.unwrap();
    assert_eq!(issue_res.status(), StatusCode::OK);
    let body_bytes = to_bytes(issue_res.into_body(), usize::MAX).await.unwrap();
    let issue_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let token = issue_json["token"].as_str().unwrap().to_string();
    assert!(token.starts_with("lumis_embed_v1."));

    // 2. Verify token via /api/auth/embed-token/verify with Authorization Bearer
    let verify_req = Request::builder()
        .method("GET")
        .uri("/api/auth/embed-token/verify")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let verify_res = app.clone().oneshot(verify_req).await.unwrap();
    assert_eq!(verify_res.status(), StatusCode::OK);
    let verify_bytes = to_bytes(verify_res.into_body(), usize::MAX).await.unwrap();
    let verify_json: Value = serde_json::from_slice(&verify_bytes).unwrap();
    assert_eq!(verify_json["valid"], true);
    assert_eq!(verify_json["user_id"], "host_usr_8812");
    assert_eq!(verify_json["username"], "sarah.connor");

    // 3. Authenticate to /api/auth/me using the Embed Token
    let me_req = Request::builder()
        .method("GET")
        .uri("/api/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let me_res = app.clone().oneshot(me_req).await.unwrap();
    assert_eq!(me_res.status(), StatusCode::OK);
    let me_bytes = to_bytes(me_res.into_body(), usize::MAX).await.unwrap();
    let me_json: Value = serde_json::from_slice(&me_bytes).unwrap();
    assert_eq!(me_json["authenticated"], true);
    assert_eq!(me_json["user"]["username"], "sarah.connor");

    // 4. Create chat session as the embedded user
    let sess_req = Request::builder()
        .method("POST")
        .uri("/api/sessions")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::json!({
            "title": "Embedded Host Session",
            "model_id": "default-model"
        }).to_string()))
        .unwrap();

    let sess_res = app.clone().oneshot(sess_req).await.unwrap();
    assert_eq!(sess_res.status(), StatusCode::CREATED);
    let sess_bytes = to_bytes(sess_res.into_body(), usize::MAX).await.unwrap();
    let sess_json: Value = serde_json::from_slice(&sess_bytes).unwrap();
    assert_eq!(sess_json["title"], "Embedded Host Session");
}

#[tokio::test]
async fn test_tampered_embed_token_rejected() {
    let app = test_app().await;

    let tampered_token = "lumis_embed_v1.eyJzdWIiOiJhZG1pbiJ9.invalidsig123";
    let req = Request::builder()
        .method("GET")
        .uri("/api/auth/embed-token/verify")
        .header(header::AUTHORIZATION, format!("Bearer {}", tampered_token))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
