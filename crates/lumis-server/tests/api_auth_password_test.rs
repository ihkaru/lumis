use std::sync::Arc;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use lumis_core::models::EnterpriseRole;
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
async fn test_login_and_password_lifecycle_verification() {
    let app = test_app().await;

    // 1. First login as admin creates super_admin with password "secret123"
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "admin",
            "password": "secret123"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let set_cookie = res.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(set_cookie.contains("lumis_user=admin"));

    // 2. Login with wrong password should fail with 401 Unauthorized
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "admin",
            "password": "wrongpassword"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 3. Login with correct password succeeds
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "admin",
            "password": "secret123"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 4. Change password with wrong old password -> 400 Bad Request
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/password")
        .header(header::COOKIE, "lumis_user=admin")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "old_password": "incorrect_old",
            "new_password": "newsecret456"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 5. Change password with valid old password -> 200 OK
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/password")
        .header(header::COOKIE, "lumis_user=admin")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "old_password": "secret123",
            "new_password": "newsecret456"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 6. Old password can no longer login -> 401 Unauthorized
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "admin",
            "password": "secret123"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 7. New password succeeds
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "admin",
            "password": "newsecret456"
        }).to_string()))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_role_demotion_single_source_of_truth() {
    let app = test_app().await;

    // 1. Initial login as admin
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "admin",
            "password": "initial_admin_pass"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["role"], "super_admin");

    // 2. Demote admin to member via admin role update
    let req = Request::builder()
        .method("POST")
        .uri("/api/admin/users/role")
        .header(header::COOKIE, "lumis_user=admin")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({
            "username": "admin",
            "role": "member"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["role"], "member");

    // 3. auth_me for admin should now return member and NOT SuperAdmin
    let req = Request::builder()
        .uri("/api/auth/me")
        .header(header::COOKIE, "lumis_user=admin")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["role"], "member");
    assert_eq!(json["role_level"], EnterpriseRole::Member.level());
}
