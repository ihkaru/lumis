use std::sync::Arc;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use lumis_core::traits::DefaultClaimTranslator;
use lumis_db::DatabaseFactory;
use lumis_server::finops::InMemoryQuotaEngine;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use tower::ServiceExt;

async fn test_app_with_custom_window(quota_engine: Arc<InMemoryQuotaEngine>) -> axum::Router {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .unwrap();
    driver.run_migrations().await.unwrap();

    let guardrail = Arc::new(
        SimdGuardrailMatcher::new(vec!["sk-test".to_string()]).unwrap(),
    );
    let claim_translator = Arc::new(DefaultClaimTranslator::new());
    let telemetry = Arc::new(lumis_server::finops::PrometheusTelemetryCollector::new());

    let state = AppState::with_options(
        driver,
        guardrail,
        claim_translator,
        quota_engine,
        telemetry,
        "x-forwarded-user".to_string(),
    );

    create_router(state)
}

#[tokio::test]
async fn test_finops_token_quota_enforcement_and_http_429() {
    let quota_engine = Arc::new(InMemoryQuotaEngine::new());
    let app = test_app_with_custom_window(quota_engine).await;

    // 1. Create a session with a Guest role user (limit 5,000 tokens)
    let create_sess_req = Request::builder()
        .method("POST")
        .uri("/api/sessions")
        .header("x-forwarded-user", "guest_user_1")
        .header("x-forwarded-groups", "tamu") // maps to Guest role
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "title": "Guest Session",
                "model_id": "gpt-4o-mini",
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(create_sess_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let sess_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let session_id = sess_json["id"].as_str().unwrap();

    // 2. Send normal message (small payload, e.g., 20 tokens) -> 200 OK
    let msg_req = Request::builder()
        .method("POST")
        .uri(format!("/api/sessions/{}/messages", session_id))
        .header("x-forwarded-user", "guest_user_1")
        .header("x-forwarded-groups", "tamu")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "content": "Pertanyaan pendek dari pengguna tamu.",
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(msg_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Send massive prompt exceeding Guest budget (5,000 tokens limit)
    // 5,000 tokens * 4 chars/token = 20,000+ chars
    let huge_content = "A".repeat(24_000);
    let quota_exceeded_req = Request::builder()
        .method("POST")
        .uri(format!("/api/sessions/{}/messages", session_id))
        .header("x-forwarded-user", "guest_user_1")
        .header("x-forwarded-groups", "tamu")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "content": huge_content,
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(quota_exceeded_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(res.headers().contains_key(header::RETRY_AFTER));

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let err_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(err_json["error"].as_str().unwrap().contains("FinOps Token Quota Exceeded"));
    assert_eq!(err_json["role"].as_str().unwrap(), "guest");
    assert_eq!(err_json["limit"].as_u64().unwrap(), 5_000);

    // 4. Verify Prometheus /metrics endpoint exposes GenAI semantic metrics
    let metrics_req = Request::builder()
        .method("GET")
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(metrics_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let metrics_text = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(metrics_text.contains("gen_ai_client_token_usage_total"));
    assert!(metrics_text.contains("gen_ai_client_operation_duration_ms_total"));
    assert!(metrics_text.contains("role=\"guest\""));

    // 5. Verify FinOps summary endpoint
    let summary_req = Request::builder()
        .method("GET")
        .uri("/api/finops/summary")
        .header("x-tenant-id", "default")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(summary_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let summary_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(summary_json.get("guest").is_some());
    assert!(summary_json.get("super_admin").is_some());
    assert_eq!(summary_json["guest"]["limit"].as_u64().unwrap(), 5_000);
}
