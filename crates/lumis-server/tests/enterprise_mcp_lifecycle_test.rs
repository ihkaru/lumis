#![deny(unsafe_code)]

use std::sync::Arc;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use chrono::Utc;
use lumis_core::models::mcp::{
    McpAuthType, McpRiskTier, McpServerCreateInput, McpServerRecord, McpToolRecord,
    McpTransportType,
};
use lumis_core::traits::{ChatRepository, DefaultClaimTranslator};
use lumis_core::EnterpriseRole;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn test_app_and_repo() -> (axum::Router, Arc<dyn ChatRepository>) {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Initialize SQLite in-memory driver");
    driver.run_migrations().await.expect("Run migrations");
    let repo = driver.repository();

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
    (create_router(state), repo)
}

#[tokio::test]
async fn test_mcp_servers_anti_ssrf_blocking() {
    let (app, _) = test_app_and_repo().await;

    // 1. Test test-endpoint with loopback IP (127.0.0.1)
    let req = Request::builder()
        .method("POST")
        .uri("/api/mcp/servers/test-endpoint")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "endpoint_url": "http://127.0.0.1:8080/mcp"
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().unwrap().contains("Anti-SSRF"));

    // 2. Test create_server with Cloud Metadata IP (169.254.169.254)
    let create_payload = McpServerCreateInput {
        name: "malicious_metadata_server".to_string(),
        display_title: "Evil Metadata Sniffer".to_string(),
        description: None,
        transport_type: McpTransportType::Sse,
        endpoint_url: Some("http://169.254.169.254/latest/meta-data".to_string()),
        command_bin: None,
        command_args: None,
        auth_type: McpAuthType::None,
        raw_credentials: None,
        risk_tier: Some(McpRiskTier::Destructive),
        scope_type: Some("system".to_string()),
        owner_user_id: None,
        is_mandatory: Some(false),
        allowed_roles: Some(vec!["admin".to_string()]),
        oauth_config: None,
    };

    let req2 = Request::builder()
        .method("POST")
        .uri("/api/mcp/servers")
        .header("x-forwarded-user", "admin_user")
        .header("x-forwarded-groups", "satker_admin")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&create_payload).unwrap()))
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::BAD_REQUEST);
    let body2 = to_bytes(res2.into_body(), usize::MAX).await.unwrap();
    let json2: Value = serde_json::from_slice(&body2).unwrap();
    assert!(json2["error"].as_str().unwrap().contains("Anti-SSRF"));
}

#[tokio::test]
async fn test_mcp_servers_rbac_and_secret_redaction_lifecycle() {
    let (app, repo) = test_app_and_repo().await;

    // 1. Regular Member attempt to create server must be HTTP 403 Forbidden
    let req_forbidden = Request::builder()
        .method("POST")
        .uri("/api/mcp/servers")
        .header("x-forwarded-user", "member_user")
        .header("x-forwarded-groups", "pegawai")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "unauthorized_server",
                "display_title": "Unauthorized Server",
                "transport_type": "stdio",
                "auth_type": "none"
            })
            .to_string(),
        ))
        .unwrap();

    let res_forbidden = app.clone().oneshot(req_forbidden).await.unwrap();
    assert_eq!(res_forbidden.status(), StatusCode::FORBIDDEN);

    // 2. Seed admin_user and MCP server with sensitive encrypted credentials
    let server_id = "mcp-srv-test-100".to_string();
    let now = Utc::now();
    let admin_user = lumis_core::models::User {
        id: "admin_user".to_string(),
        username: "admin_user".to_string(),
        email: Some("admin@enterprise.internal".to_string()),
        role: EnterpriseRole::TenantAdmin,
        tenant_id: Some("default".to_string()),
        preferences: json!({}),
        is_active: true,
        password_hash: None,
        created_at: now,
        updated_at: now,
    };
    repo.upsert_user(&admin_user).await.unwrap();

    let record = McpServerRecord {
        id: server_id.clone(),
        tenant_id: "default".to_string(),
        created_by: "admin_user".to_string(),
        name: "enterprise_postgres_mcp".to_string(),
        display_title: "Enterprise Ledger MCP".to_string(),
        description: Some("Production relational ledger tools".to_string()),
        transport_type: McpTransportType::Sse,
        endpoint_url: Some("https://mcp.enterprise.internal/sse".to_string()),
        command_bin: None,
        command_args: None,
        auth_type: McpAuthType::Bearer,
        encrypted_credentials: Some("v1:U2VjcmV0Tm9uY2Ux:Q2lwaGVydGV4dERhdGE=".to_string()),
        is_enabled: true,
        risk_tier: McpRiskTier::Mutation,
        scope_type: "system".to_string(),
        owner_user_id: None,
        is_mandatory: false,
        allowed_roles: vec!["admin".to_string(), "member".to_string()],
        oauth_config: None,
        created_at: now,
        updated_at: now,
    };
    repo.create_mcp_server(&record).await.unwrap();

    let tool_record = McpToolRecord {
        id: "mcp-tool-101".to_string(),
        server_id: server_id.clone(),
        name: "query_general_ledger".to_string(),
        description: Some("Query ledger transactions".to_string()),
        input_schema_json: json!({"type": "object"}).to_string(),
        risk_tier: McpRiskTier::Safe,
        requires_approval: false,
        updated_at: now,
    };
    repo.sync_mcp_tools(&server_id, &[tool_record]).await.unwrap();

    // 3. GET /api/mcp/servers - verify credentials are NULL / REDACTED
    let req_list = Request::builder()
        .method("GET")
        .uri("/api/mcp/servers")
        .header("x-forwarded-user", "admin_user")
        .header("x-forwarded-groups", "satker_admin")
        .body(Body::empty())
        .unwrap();

    let res_list = app.clone().oneshot(req_list).await.unwrap();
    assert_eq!(res_list.status(), StatusCode::OK);
    let body_list = to_bytes(res_list.into_body(), usize::MAX).await.unwrap();
    let json_list: Value = serde_json::from_slice(&body_list).unwrap();
    let servers = json_list["servers"].as_array().unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0]["name"], "enterprise_postgres_mcp");
    assert!(servers[0]["encrypted_credentials"].is_null()); // Verified: never leaked!

    // 4. GET /api/mcp/servers/{id} - verify details + tools
    let req_get = Request::builder()
        .method("GET")
        .uri(format!("/api/mcp/servers/{}", server_id))
        .header("x-forwarded-user", "admin_user")
        .header("x-forwarded-groups", "satker_admin")
        .body(Body::empty())
        .unwrap();

    let res_get = app.clone().oneshot(req_get).await.unwrap();
    assert_eq!(res_get.status(), StatusCode::OK);
    let body_get = to_bytes(res_get.into_body(), usize::MAX).await.unwrap();
    let json_get: Value = serde_json::from_slice(&body_get).unwrap();
    assert_eq!(json_get["server"]["name"], "enterprise_postgres_mcp");
    assert!(json_get["server"]["encrypted_credentials"].is_null());
    let tools = json_get["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "query_general_ledger");

    // 5. PUT /api/mcp/servers/{id} - update title & enabled
    let req_update = Request::builder()
        .method("PUT")
        .uri(format!("/api/mcp/servers/{}", server_id))
        .header("x-forwarded-user", "admin_user")
        .header("x-forwarded-groups", "satker_admin")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "display_title": "Enterprise Ledger (Updated)",
                "is_enabled": false
            })
            .to_string(),
        ))
        .unwrap();

    let res_update = app.clone().oneshot(req_update).await.unwrap();
    assert_eq!(res_update.status(), StatusCode::OK);
    let body_update = to_bytes(res_update.into_body(), usize::MAX).await.unwrap();
    let json_update: Value = serde_json::from_slice(&body_update).unwrap();
    assert_eq!(json_update["server"]["display_title"], "Enterprise Ledger (Updated)");
    assert_eq!(json_update["server"]["is_enabled"], false);

    // 6. DELETE /api/mcp/servers/{id} - verify deletion
    let req_del = Request::builder()
        .method("DELETE")
        .uri(format!("/api/mcp/servers/{}", server_id))
        .header("x-forwarded-user", "admin_user")
        .header("x-forwarded-groups", "satker_admin")
        .body(Body::empty())
        .unwrap();

    let res_del = app.clone().oneshot(req_del).await.unwrap();
    assert_eq!(res_del.status(), StatusCode::OK);

    // Verify server no longer exists
    let req_get_after = Request::builder()
        .method("GET")
        .uri(format!("/api/mcp/servers/{}", server_id))
        .header("x-forwarded-user", "admin_user")
        .header("x-forwarded-groups", "satker_admin")
        .body(Body::empty())
        .unwrap();

    let res_get_after = app.oneshot(req_get_after).await.unwrap();
    assert_eq!(res_get_after.status(), StatusCode::NOT_FOUND);
}
