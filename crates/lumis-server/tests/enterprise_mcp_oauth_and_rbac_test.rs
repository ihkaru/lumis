#![deny(unsafe_code)]

use std::sync::Arc;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use chrono::Utc;
use lumis_core::models::mcp::{
    McpAuthType, McpOAuthConfig, McpRiskTier, McpServerCreateInput, McpTransportType,
};
use lumis_core::models::user::{EnterpriseRole, User};
use lumis_core::traits::{ChatRepository, DefaultClaimTranslator};
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn setup_test_app() -> (axum::Router, Arc<dyn ChatRepository>) {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Initialize SQLite in-memory driver");
    driver.run_migrations().await.expect("Run migrations");
    let repo = driver.repository();

    let now = Utc::now();
    // Seed test users
    let admin_user = User {
        id: "admin-usr-01".to_string(),
        username: "security_officer".to_string(),
        email: Some("soc@enterprise.internal".to_string()),
        role: EnterpriseRole::TenantAdmin,
        tenant_id: Some("default".to_string()),
        preferences: json!({}),
        is_active: true,
        password_hash: None,
        created_at: now,
        updated_at: now,
    };
    let regular_user = User {
        id: "member-usr-01".to_string(),
        username: "regular_employee".to_string(),
        email: Some("emp@enterprise.internal".to_string()),
        role: EnterpriseRole::Member,
        tenant_id: Some("default".to_string()),
        preferences: json!({}),
        is_active: true,
        password_hash: None,
        created_at: now,
        updated_at: now,
    };
    let _ = repo.upsert_user(&admin_user).await.unwrap();
    let _ = repo.upsert_user(&regular_user).await.unwrap();

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

    let router = create_router(state);
    (router, repo)
}

#[tokio::test]
async fn test_ciso_mandatory_mcp_cannot_be_disabled_by_user() {
    let (app, _) = setup_test_app().await;

    // 1. Admin creates a Mandatory MCP server (e.g. Enterprise DLP Sentinel)
    let mandatory_server = McpServerCreateInput {
        name: "dlp_sentinel".to_string(),
        display_title: "Enterprise DLP Sentinel".to_string(),
        description: Some("Mandatory compliance inspection".to_string()),
        transport_type: McpTransportType::Stdio,
        endpoint_url: None,
        command_bin: Some("/usr/bin/dlp-mcp".to_string()),
        command_args: None,
        auth_type: McpAuthType::None,
        raw_credentials: None,
        risk_tier: Some(McpRiskTier::Safe),
        scope_type: Some("system".to_string()),
        owner_user_id: None,
        is_mandatory: Some(true),
        allowed_roles: Some(vec!["admin".to_string(), "member".to_string()]),
        oauth_config: None,
        ssrf_policy: None,
        egress_allowlist: None,
        sandbox_type: None,
        max_memory_mb: None,
        timeout_seconds: None,
        dlp_enabled: None,
        dlp_rules: None,
        rate_limit_rpm: None,
        daily_request_limit: None,
        kms_provider: None,
        kms_key_id: None,
    };

    let req = Request::builder()
        .method("POST")
        .uri("/api/mcp/servers")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-user", "security_officer")
        .body(Body::from(serde_json::to_vec(&mandatory_server).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_resp: Value = serde_json::from_slice(&body).unwrap();
    let server_id = json_resp["server"]["id"].as_str().unwrap().to_string();

    // 2. Regular user tries to disable the mandatory server (Toggle Off)
    let toggle_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/mcp/preferences/{}", server_id))
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-user", "regular_employee")
        .body(Body::from(serde_json::to_vec(&json!({ "is_enabled": false })).unwrap()))
        .unwrap();

    let toggle_res = app.clone().oneshot(toggle_req).await.unwrap();
    // Invariant: Must fail with HTTP 400 Bad Request
    assert_eq!(toggle_res.status(), StatusCode::BAD_REQUEST);
    let toggle_body = to_bytes(toggle_res.into_body(), usize::MAX).await.unwrap();
    let err_json: Value = serde_json::from_slice(&toggle_body).unwrap();
    assert!(err_json["error"].as_str().unwrap().contains("Mandatory (CISO Enforced)"));
}

#[tokio::test]
async fn test_mcp_oauth_pkce_handshake_and_token_vault_encryption() {
    let (app, repo) = setup_test_app().await;

    // 1. Create an OAuth-protected MCP Server
    let oauth_server = McpServerCreateInput {
        name: "jira_ledger".to_string(),
        display_title: "Corporate Jira & Ledger".to_string(),
        description: Some("OAuth 2.1 protected integration".to_string()),
        transport_type: McpTransportType::Stdio,
        endpoint_url: Some("https://jira.enterprise.com/mcp".to_string()),
        command_bin: None,
        command_args: None,
        auth_type: McpAuthType::OAuth2,
        raw_credentials: None,
        risk_tier: Some(McpRiskTier::Safe),
        scope_type: Some("system".to_string()),
        owner_user_id: None,
        is_mandatory: Some(false),
        allowed_roles: Some(vec!["admin".to_string(), "member".to_string()]),
        oauth_config: Some(McpOAuthConfig {
            client_id: "lumis-client-jira".to_string(),
            client_secret_encrypted: None,
            authorization_endpoint: "https://auth.enterprise.com/oauth/authorize".to_string(),
            token_endpoint: "https://auth.enterprise.com/oauth/token".to_string(),
            scopes: vec!["read:issues".to_string(), "write:issues".to_string()],
            resource_indicator: Some("https://jira.enterprise.com/mcp".to_string()),
            registration_mode: "pre_registered".to_string(),
        }),
        ssrf_policy: None,
        egress_allowlist: None,
        sandbox_type: None,
        max_memory_mb: None,
        timeout_seconds: None,
        dlp_enabled: None,
        dlp_rules: None,
        rate_limit_rpm: None,
        daily_request_limit: None,
        kms_provider: None,
        kms_key_id: None,
    };

    let create_req = Request::builder()
        .method("POST")
        .uri("/api/mcp/servers")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-user", "security_officer")
        .body(Body::from(serde_json::to_vec(&oauth_server).unwrap()))
        .unwrap();

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);
    let body = to_bytes(create_res.into_body(), usize::MAX).await.unwrap();
    let json_resp: Value = serde_json::from_slice(&body).unwrap();
    let server_id = json_resp["server"]["id"].as_str().unwrap().to_string();

    // 2. Initiate PKCE OAuth flow for regular user
    let auth_req = Request::builder()
        .method("POST")
        .uri(format!("/api/mcp/servers/{}/oauth/authorize", server_id))
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-user", "regular_employee")
        .body(Body::from(serde_json::to_vec(&json!({
            "redirect_uri": "https://lumis.enterprise.com/api/mcp/callback",
            "state": "test_state_123"
        })).unwrap()))
        .unwrap();

    let auth_res = app.clone().oneshot(auth_req).await.unwrap();
    assert_eq!(auth_res.status(), StatusCode::OK);
    let auth_body = to_bytes(auth_res.into_body(), usize::MAX).await.unwrap();
    let auth_json: Value = serde_json::from_slice(&auth_body).unwrap();

    let auth_url = auth_json["authorization_url"].as_str().unwrap();
    assert!(auth_url.contains("code_challenge_method=S256"));
    assert!(auth_url.contains("client_id=lumis-client-jira"));
    assert!(auth_url.contains("resource=https%3A%2F%2Fjira.enterprise.com%2Fmcp"));

    // 3. Complete Callback and verify token storage
    let callback_req = Request::builder()
        .method("POST")
        .uri(format!("/api/mcp/servers/{}/oauth/callback", server_id))
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-user", "regular_employee")
        .body(Body::from(serde_json::to_vec(&json!({
            "access_token": "secret_oauth_access_token_12345",
            "refresh_token": "secret_oauth_refresh_token_67890",
            "token_type": "Bearer",
            "expires_in": 3600
        })).unwrap()))
        .unwrap();

    let callback_res = app.clone().oneshot(callback_req).await.unwrap();
    assert_eq!(callback_res.status(), StatusCode::OK);

    // 4. Verify in relational vault: tokens MUST be encrypted with AES-256-GCM (starts with "v1:")
    let raw_token = repo.get_user_mcp_oauth_token("member-usr-01", &server_id).await.unwrap();
    assert!(raw_token.is_some());
    let token_record = raw_token.unwrap();
    assert!(token_record.access_token_encrypted.starts_with("v1:"));
    assert_ne!(token_record.access_token_encrypted, "secret_oauth_access_token_12345");
}

#[tokio::test]
async fn test_role_based_mcp_server_filtering_and_preferences() {
    let (app, _) = setup_test_app().await;

    // 1. Admin creates a server restricted only to 'admin'
    let admin_only_server = McpServerCreateInput {
        name: "k8s_cluster_ops".to_string(),
        display_title: "Kubernetes Cluster Operations".to_string(),
        description: Some("Admin only infrastructure controller".to_string()),
        transport_type: McpTransportType::Stdio,
        endpoint_url: None,
        command_bin: Some("/usr/bin/k8s-mcp".to_string()),
        command_args: None,
        auth_type: McpAuthType::None,
        raw_credentials: None,
        risk_tier: Some(McpRiskTier::Destructive),
        scope_type: Some("system".to_string()),
        owner_user_id: None,
        is_mandatory: Some(false),
        allowed_roles: Some(vec!["tenant_admin".to_string(), "super_admin".to_string()]),
        oauth_config: None,
        ssrf_policy: None,
        egress_allowlist: None,
        sandbox_type: None,
        max_memory_mb: None,
        timeout_seconds: None,
        dlp_enabled: None,
        dlp_rules: None,
        rate_limit_rpm: None,
        daily_request_limit: None,
        kms_provider: None,
        kms_key_id: None,
    };

    let create_req = Request::builder()
        .method("POST")
        .uri("/api/mcp/servers")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-user", "security_officer")
        .body(Body::from(serde_json::to_vec(&admin_only_server).unwrap()))
        .unwrap();

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    // 2. Regular employee fetches preferences -> k8s server MUST NOT be visible
    let user_pref_req = Request::builder()
        .method("GET")
        .uri("/api/mcp/preferences")
        .header("x-forwarded-user", "regular_employee")
        .body(Body::empty())
        .unwrap();

    let user_pref_res = app.clone().oneshot(user_pref_req).await.unwrap();
    assert_eq!(user_pref_res.status(), StatusCode::OK);
    let user_body = to_bytes(user_pref_res.into_body(), usize::MAX).await.unwrap();
    let user_json: Value = serde_json::from_slice(&user_body).unwrap();
    let servers = user_json["servers"].as_array().unwrap();
    assert!(!servers.iter().any(|s| s["name"] == "k8s_cluster_ops"));

    // 3. Security officer fetches preferences -> k8s server MUST be visible
    let admin_pref_req = Request::builder()
        .method("GET")
        .uri("/api/mcp/preferences")
        .header("x-forwarded-user", "security_officer")
        .body(Body::empty())
        .unwrap();

    let admin_pref_res = app.clone().oneshot(admin_pref_req).await.unwrap();
    assert_eq!(admin_pref_res.status(), StatusCode::OK);
    let admin_body = to_bytes(admin_pref_res.into_body(), usize::MAX).await.unwrap();
    let admin_json: Value = serde_json::from_slice(&admin_body).unwrap();
    let admin_servers = admin_json["servers"].as_array().unwrap();
    assert!(admin_servers.iter().any(|s| s["name"] == "k8s_cluster_ops"));
}
