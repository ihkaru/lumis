#![deny(unsafe_code)]

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
async fn test_mcp_ema_role_gated_tool_discovery() {
    let app = test_app().await;

    // 1. Guest user lists tools -> neither system_status (Member) nor database_audit (TenantAdmin) visible
    let guest_req = Request::builder()
        .uri("/api/mcp/tools")
        .header("x-forwarded-user", "guest_user")
        .header("x-forwarded-groups", "tamu") // Maps to Guest
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(guest_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let tools: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(tools.len(), 0, "Guest must see zero tools");

    // 2. Member user lists tools -> sees system_status only
    let member_req = Request::builder()
        .uri("/api/mcp/tools")
        .header("x-forwarded-user", "regular_staff")
        .header("x-forwarded-groups", "pegawai") // Maps to Member
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(member_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let tools: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(tools.len(), 1, "Member must only see member-level tools");
    assert_eq!(tools[0]["name"], "system_status");

    // 3. TenantAdmin lists tools -> sees both system_status and database_audit
    let admin_req = Request::builder()
        .uri("/api/mcp/tools")
        .header("x-forwarded-user", "satker_lead")
        .header("x-forwarded-groups", "manager") // Maps to TenantAdmin
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(admin_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let tools: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(tools.len(), 2, "TenantAdmin must see both member and admin tools");
}

#[tokio::test]
async fn test_mcp_ema_role_gated_tool_execution() {
    let app = test_app().await;

    // 1. Member tries to invoke administrative database_audit tool -> MUST BE REJECTED 403 FORBIDDEN
    let forbidden_call_req = Request::builder()
        .method("POST")
        .uri("/api/mcp/tools/call")
        .header("x-forwarded-user", "regular_staff")
        .header("x-forwarded-groups", "pegawai") // Maps to Member
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "tool_name": "database_audit",
                "arguments": { "audit_scope": "security_keys" },
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(forbidden_call_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN, "Member must not invoke TenantAdmin tool");
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().unwrap().contains("lacks clearance"));

    // 2. TenantAdmin invokes administrative database_audit tool -> SUCCEEDS 200 OK
    let authorized_call_req = Request::builder()
        .method("POST")
        .uri("/api/mcp/tools/call")
        .header("x-forwarded-user", "satker_lead")
        .header("x-forwarded-groups", "manager") // Maps to TenantAdmin
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "tool_name": "database_audit",
                "arguments": { "audit_scope": "security_keys" },
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(authorized_call_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["is_error"], false);
    assert!(json["content"].as_str().unwrap().contains("COMPLIANT_ZERO_MOCK"));

    // 3. Member invokes accessible system_status tool -> SUCCEEDS 200 OK
    let member_valid_call = Request::builder()
        .method("POST")
        .uri("/api/mcp/tools/call")
        .header("x-forwarded-user", "regular_staff")
        .header("x-forwarded-groups", "pegawai")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "tool_name": "system_status",
                "arguments": {},
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(member_valid_call).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["content"].as_str().unwrap().contains("lumis-native-rust"));
}

#[tokio::test]
async fn test_mcp_asynchronous_tasks_lifecycle() {
    let app = test_app().await;

    // 1. Spawn async task for system_status
    let spawn_req = Request::builder()
        .method("POST")
        .uri("/api/mcp/tasks")
        .header("x-forwarded-user", "regular_staff")
        .header("x-forwarded-groups", "pegawai")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "tool_name": "system_status",
                "arguments": {},
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(spawn_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let task_json: Value = serde_json::from_slice(&body).unwrap();

    let task_id = task_json["task_id"].as_str().unwrap();
    assert!(task_id.starts_with("task-"));
    assert_eq!(task_json["tool_name"], "system_status");
    assert_eq!(task_json["state"], "completed");

    // 2. Poll task status via GET /api/mcp/tasks/{task_id}
    let poll_req = Request::builder()
        .uri(format!("/api/mcp/tasks/{}", task_id))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(poll_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let polled_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(polled_json["task_id"], task_id);
    assert_eq!(polled_json["state"], "completed");
    assert!(polled_json["result"]["content"].as_str().unwrap().contains("lumis-native-rust"));

    // 3. Query non-existent task returns 404
    let not_found_req = Request::builder()
        .uri("/api/mcp/tasks/task-non-existent-999")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(not_found_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
