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
        SimdGuardrailMatcher::new(vec!["sk-ant-".to_string(), "DROP TABLE".to_string()])
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
async fn test_health_and_db_health_endpoints() {
    let app = test_app().await;

    // 1. GET /health
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. GET /health/db
    let req = Request::builder()
        .uri("/health/db")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["dialect"], "sqlite");
    assert_eq!(json["is_alive"], true);
}

#[tokio::test]
async fn test_api_admin_roles_hierarchy() {
    let app = test_app().await;

    let req = Request::builder()
        .uri("/api/admin/roles")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let roles: Value = serde_json::from_slice(&bytes).unwrap();
    let arr = roles.as_array().expect("Array of roles");
    assert_eq!(arr.len(), 5);
}

#[tokio::test]
async fn test_auth_me_scenarios() {
    let app = test_app().await;

    // Scenario A: Anonymous -> 401 Unauthorized
    let req = Request::builder()
        .uri("/api/auth/me")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Scenario B: Staff user with SAML / AD headers
    let req = Request::builder()
        .uri("/api/auth/me")
        .header("x-forwarded-user", "john_doe")
        .header("x-forwarded-email", "john@enterprise.com")
        .header("x-forwarded-groups", "pegawai, internal")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get("x-forwarded-role").unwrap(), "member");
    assert_eq!(res.headers().get("x-forwarded-tier").unwrap(), "tier_standard");
    assert_eq!(res.headers().get("x-user-role-level").unwrap(), "2");

    // Scenario C: CISO / Security Officer -> SuperAdmin
    let req = Request::builder()
        .uri("/api/auth/me")
        .header("x-forwarded-user", "ciso_admin")
        .header("x-forwarded-groups", "ciso")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get("x-forwarded-role").unwrap(), "super_admin");
    assert_eq!(res.headers().get("x-forwarded-tier").unwrap(), "tier_reasoning");
    assert_eq!(res.headers().get("x-user-role-level").unwrap(), "5");
}

#[tokio::test]
async fn test_model_clearance_pep_enforcement() {
    let app = test_app().await;

    // Guest user trying to create session with TierReasoning model (deepseek-r1) -> 403 FORBIDDEN
    let req = Request::builder()
        .method("POST")
        .uri("/api/sessions")
        .header("x-forwarded-user", "guest_auditor")
        .header("x-forwarded-groups", "tamu")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "title": "Reasoning Attempt",
                "model_id": "deepseek-r1",
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // Guest user creating session with TierBasic model (gpt-4o-mini) -> 201 CREATED
    let req = Request::builder()
        .method("POST")
        .uri("/api/sessions")
        .header("x-forwarded-user", "guest_auditor")
        .header("x-forwarded-groups", "tamu")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "title": "Basic Model Session",
                "model_id": "gpt-4o-mini",
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}
