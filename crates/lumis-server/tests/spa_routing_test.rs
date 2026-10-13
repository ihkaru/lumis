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

    // Point STATIC_DIR to ui/dist for test verification
    std::env::set_var("STATIC_DIR", "ui/dist");
    create_router(state)
}

#[tokio::test]
async fn test_spa_deep_linking_chat_and_login_return_200_html() {
    let app = test_app().await;

    // 1. Direct deep link: /chat/a310b9fa-cb51-4b57-b1f3-faafcef8713e
    let req = Request::builder()
        .uri("/chat/a310b9fa-cb51-4b57-b1f3-faafcef8713e")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let content_type = res.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
    assert!(content_type.contains("text/html"));

    let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let body = String::from_utf8_lossy(&bytes);
    assert!(body.contains("<!doctype html>"));

    // 2. Direct deep link: /login
    let req_login = Request::builder()
        .uri("/login")
        .body(Body::empty())
        .unwrap();

    let res_login = app.clone().oneshot(req_login).await.unwrap();
    assert_eq!(res_login.status(), StatusCode::OK);
    let content_type_login = res_login.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
    assert!(content_type_login.contains("text/html"));
}

#[tokio::test]
async fn test_spa_asset_isolation_and_api_guard() {
    let app = test_app().await;

    // 1. Existing physical asset must return 200 with proper asset mime type
    let req_asset = Request::builder()
        .uri("/assets/index-_r_61Sps.js")
        .body(Body::empty())
        .unwrap();

    let res_asset = app.clone().oneshot(req_asset).await.unwrap();
    assert_eq!(res_asset.status(), StatusCode::OK);

    // 2. Missing asset with extension must return 404, NEVER index.html (MIME protection)
    let req_missing_asset = Request::builder()
        .uri("/assets/nonexistent-chunk-999.js")
        .body(Body::empty())
        .unwrap();

    let res_missing = app.clone().oneshot(req_missing_asset).await.unwrap();
    assert_eq!(res_missing.status(), StatusCode::NOT_FOUND);

    // 3. Unmatched API route must return 404 JSON, NEVER HTML
    let req_api = Request::builder()
        .uri("/api/nonexistent/endpoint")
        .body(Body::empty())
        .unwrap();

    let res_api = app.clone().oneshot(req_api).await.unwrap();
    assert_eq!(res_api.status(), StatusCode::NOT_FOUND);
    let bytes = to_bytes(res_api.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"], "Endpoint not found");

    // 4. Non-GET/HEAD request on client route must return 405 Method Not Allowed
    let req_post = Request::builder()
        .method("POST")
        .uri("/chat/a310b9fa-cb51-4b57-b1f3-faafcef8713e")
        .body(Body::empty())
        .unwrap();

    let res_post = app.clone().oneshot(req_post).await.unwrap();
    assert_eq!(res_post.status(), StatusCode::METHOD_NOT_ALLOWED);
}
