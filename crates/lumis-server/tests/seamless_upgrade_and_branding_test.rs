use std::sync::Arc;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
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
async fn test_runtime_branding_zero_recompile() {
    // 1. Test default branding
    let app = test_app().await;
    let req = Request::builder()
        .uri("/api/config/branding")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["app_title"], "Lumis");
    assert_eq!(body["upgrade_channel"], "stable");
    assert!(body["version"].is_string());

    // 2. Test runtime brand override via environment variables (Zero-Recompile White-Labeling)
    std::env::set_var("LUMIS_BRAND_TITLE", "Bank Mandiri Enterprise AI");
    std::env::set_var("LUMIS_BRAND_LOGO_URL", "https://bankmandiri.co.id/logo.svg");
    std::env::set_var("LUMIS_BRAND_THEME_PRIMARY", "#0052cc");
    std::env::set_var("LUMIS_UPGRADE_CHANNEL", "v1.2-enterprise");

    let req2 = Request::builder()
        .uri("/api/config/branding")
        .body(Body::empty())
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);

    let bytes2 = to_bytes(res2.into_body(), 1024 * 1024).await.unwrap();
    let body2: Value = serde_json::from_slice(&bytes2).unwrap();
    assert_eq!(body2["app_title"], "Bank Mandiri Enterprise AI");
    assert_eq!(body2["logo_url"], "https://bankmandiri.co.id/logo.svg");
    assert_eq!(body2["primary_color"], "#0052cc");
    assert_eq!(body2["upgrade_channel"], "v1.2-enterprise");

    // Clean up env vars
    std::env::remove_var("LUMIS_BRAND_TITLE");
    std::env::remove_var("LUMIS_BRAND_LOGO_URL");
    std::env::remove_var("LUMIS_BRAND_THEME_PRIMARY");
    std::env::remove_var("LUMIS_UPGRADE_CHANNEL");
}

#[tokio::test]
async fn test_system_doctor_pre_flight_upgrade_diagnostic() {
    let app = test_app().await;

    let req = Request::builder()
        .uri("/api/system/doctor")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let report: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(report["database_healthy"], true);
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["pending_migrations"], 0);
    assert_eq!(report["ready_for_upgrade"], true);
    assert_eq!(report["deprecated_configs"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_idempotent_expand_and_contract_migration_safety() {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Initialize SQLite in-memory driver");

    // 1st run of migrations
    let r1 = driver.run_migrations().await;
    assert!(r1.is_ok(), "First migration run must succeed");

    // 2nd run of migrations (simulates pod restart or new container rolling deployment)
    let r2 = driver.run_migrations().await;
    assert!(r2.is_ok(), "Second migration run must be idempotent and non-destructive");

    // 3rd run (simulates horizontal pod scaling)
    let r3 = driver.run_migrations().await;
    assert!(r3.is_ok(), "Third migration run must remain idempotent");

    let is_healthy = driver.health().await.unwrap().is_alive;
    assert!(is_healthy, "Driver must be healthy throughout all rolling migration cycles");
}

#[tokio::test]
async fn test_dynamic_models_discovery_and_catalog() {
    let app = test_app().await;

    // Test GET /api/models returns valid model list with capabilities and clearance
    let req = Request::builder()
        .uri("/api/models")
        .header("x-forwarded-user", "pegawai_bps")
        .header("x-forwarded-roles", "member")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let models: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(models.is_array());
    let list = models.as_array().unwrap();
    assert!(!list.is_empty(), "Must return at least default models");

    // Check first model has required contract fields
    let first = &list[0];
    assert!(first["id"].is_string());
    assert!(first["name"].is_string());
    assert!(first["tier"].is_string());
    assert!(first["provider"].is_string());
    assert!(first["can_access"].is_boolean());

    // Check alias route /api/v1/models works identically
    let req2 = Request::builder()
        .uri("/api/v1/models")
        .body(Body::empty())
        .unwrap();
    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
}

