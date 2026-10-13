use std::sync::Arc;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use lumis_core::models::prompt_cache::PromptLayout;
use lumis_core::models::ChatMessagePayload;
use lumis_core::traits::DefaultClaimTranslator;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use tower::ServiceExt;

async fn test_app() -> axum::Router {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Initialize SQLite in-memory driver");
    driver.run_migrations().await.expect("Run migrations");

    let guardrail = Arc::new(
        SimdGuardrailMatcher::new(vec!["sk-test-secret".to_string()])
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
async fn test_prompt_layout_stable_prefix_invariance_and_anthropic_breakpoint() {
    let system_instructions = "You are an enterprise assistant. Follow strict compliance guidelines.";
    let tool_schemas = serde_json::json!([
        { "name": "query_db", "parameters": { "type": "object" } }
    ]).to_string();

    // Turn 1 with initial dynamic timestamp
    let layout_turn1 = PromptLayout::new(
        system_instructions,
        &tool_schemas,
        vec![ChatMessagePayload::new("user", "Hello assistant")],
        Some("Timestamp: 2026-10-12T19:00:00.000Z".to_string()),
    );

    // Turn 2 with different timestamp (LibreChat#16877 regression scenario)
    let layout_turn2 = PromptLayout::new(
        system_instructions,
        &tool_schemas,
        vec![
            ChatMessagePayload::new("user", "Hello assistant"),
            ChatMessagePayload::new("assistant", "Greetings! How can I help?"),
            ChatMessagePayload::new("user", "Can you query the database?"),
        ],
        Some("Timestamp: 2026-10-12T19:00:45.123Z".to_string()),
    );

    // Prefix hashes MUST be 100% identical byte-for-byte to trigger provider KV caching
    let hash1 = layout_turn1.stable_prefix_hash();
    let hash2 = layout_turn2.stable_prefix_hash();
    assert_eq!(hash1, hash2, "Stable prefix hash must remain invariant across turns");

    // Verify Anthropic ephemeral cache control block
    let anthropic_block = layout_turn1.to_anthropic_system_block();
    assert_eq!(anthropic_block[0]["type"], "text");
    assert_eq!(anthropic_block[0]["cache_control"]["type"], "ephemeral");
}

#[tokio::test]
async fn test_start_generation_idempotency_ahead_of_quota_deduction() {
    let app = test_app().await;

    // Create session first
    let create_sess_req = Request::builder()
        .method("POST")
        .uri("/api/sessions")
        .header("content-type", "application/json")
        .header("x-forwarded-user", "alice")
        .header("x-forwarded-groups", "power_user")
        .body(Body::from(
            serde_json::json!({
                "title": "Idempotency Test Session",
                "model_id": "claude-3-5-sonnet",
            })
            .to_string(),
        ))
        .unwrap();

    let sess_res = app.clone().oneshot(create_sess_req).await.unwrap();
    assert_eq!(sess_res.status(), StatusCode::CREATED);
    let sess_bytes = to_bytes(sess_res.into_body(), usize::MAX).await.unwrap();
    let sess_json: serde_json::Value = serde_json::from_slice(&sess_bytes).unwrap();
    let session_id = sess_json["id"].as_str().unwrap();

    let req_body = serde_json::json!({
        "content": "Execute expensive enterprise calculation",
        "model_id": "claude-3-5-sonnet"
    }).to_string();

    // First request with idempotency key
    let req1 = Request::builder()
        .method("POST")
        .uri(format!("/api/sessions/{}/messages", session_id))
        .header("content-type", "application/json")
        .header("x-forwarded-user", "alice")
        .header("x-forwarded-groups", "power_user")
        .header("idempotency-key", "req-uuid-12345")
        .body(Body::from(req_body.clone()))
        .unwrap();

    let resp1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Immediate duplicate retry request with identical idempotency key (LibreChat#14349)
    let req2 = Request::builder()
        .method("POST")
        .uri(format!("/api/sessions/{}/messages", session_id))
        .header("content-type", "application/json")
        .header("x-forwarded-user", "alice")
        .header("x-forwarded-groups", "power_user")
        .header("idempotency-key", "req-uuid-12345")
        .body(Body::from(req_body))
        .unwrap();

    let resp2 = app.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);

    let body_bytes = to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
    let json_val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    // Must be returned as completed_cached without re-charging or 429
    assert_eq!(json_val["status"], "completed_cached");
    assert_eq!(json_val["idempotency_key"], "req-uuid-12345");
}

#[tokio::test]
async fn test_mcp_catalog_etag_and_304_not_modified() {
    let app = test_app().await;

    // Initial fetch to obtain ETag (LibreChat#15757)
    let req1 = Request::builder()
        .method("GET")
        .uri("/api/mcp/tools")
        .header("x-forwarded-user", "bob")
        .header("x-forwarded-roles", "Member")
        .body(Body::empty())
        .unwrap();

    let resp1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);
    let etag_header = resp1.headers().get("etag").unwrap().to_str().unwrap().to_string();
    assert!(etag_header.starts_with("\"mcp-v"));

    // Conditional fetch with If-None-Match matching current ETag
    let req2 = Request::builder()
        .method("GET")
        .uri("/api/mcp/tools")
        .header("x-forwarded-user", "bob")
        .header("x-forwarded-roles", "Member")
        .header("if-none-match", &etag_header)
        .body(Body::empty())
        .unwrap();

    let resp2 = app.oneshot(req2).await.unwrap();
    // Must return 304 Not Modified to completely eliminate redundant payload transfer
    assert_eq!(resp2.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(resp2.headers().get("etag").unwrap(), etag_header.as_str());
}
