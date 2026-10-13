#![deny(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use lumis_core::models::mcp::{
    McpAuthType, McpRiskTier, McpServerCreateInput, McpServerRecord, McpServerUpdateInput,
    McpToolRecord, McpTransportType,
};
use lumis_core::models::EnterpriseRole;
use lumis_core::security::{encrypt_mcp_secret, validate_mcp_endpoint_url};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::mcp::{McpHttpClient, RemoteMcpToolProxy};
use crate::state::AppState;

fn get_master_key() -> Vec<u8> {
    std::env::var("LUMIS_MCP_ENCRYPTION_KEY")
        .or_else(|_| std::env::var("LUMIS_SECRET_KEY"))
        .unwrap_or_else(|_| "lumis-default-master-key-32b-secret!".to_string())
        .into_bytes()
}

async fn extract_tenant_and_role(headers: &HeaderMap, state: &AppState) -> (String, String, EnterpriseRole) {
    let mut header_map = HashMap::new();
    for (k, v) in headers.iter() {
        if let Ok(str_val) = v.to_str() {
            header_map.insert(k.as_str().to_lowercase(), str_val.to_string());
        }
    }
    if let Some(val) = headers.get(&state.trusted_header).and_then(|v| v.to_str().ok()) {
        header_map.insert("x-forwarded-user".to_string(), val.to_string());
    }

    if let Ok(claims) = state.claim_translator.extract_claims(&header_map) {
        let claim_role = if !claims.roles.is_empty() {
            Some(state.claim_translator.resolve_effective_role(&claims))
        } else {
            None
        };

        let (user_id, role) = match state.repo.get_user_by_username(&claims.username).await {
            Ok(Some(u)) => {
                let r = claim_role.unwrap_or(u.role);
                (u.id, r)
            }
            _ => {
                let r = claim_role.unwrap_or(EnterpriseRole::Member);
                (claims.username.clone(), r)
            }
        };
        (claims.tenant_id.unwrap_or_else(|| "default".to_string()), user_id, role)
    } else {
        ("default".to_string(), "anonymous".to_string(), EnterpriseRole::Guest)
    }
}

/// GET /api/mcp/servers
pub async fn list_servers(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let (tenant_id, _, _) = extract_tenant_and_role(&headers, &state).await;
    match state.repo.list_mcp_servers(&tenant_id).await {
        Ok(mut servers) => {
            for s in &mut servers {
                s.encrypted_credentials = None;
            }
            Json(json!({ "servers": servers })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// POST /api/mcp/servers
pub async fn create_server(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<McpServerCreateInput>,
) -> impl IntoResponse {
    let (tenant_id, user_id, role) = extract_tenant_and_role(&headers, &state).await;
    if !role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Insufficient permissions" }))).into_response();
    }
    if payload.name.trim().is_empty() || payload.display_title.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Name and title are required" }))).into_response();
    }

    let endpoint_url = payload.endpoint_url.clone().unwrap_or_default();
    if payload.transport_type != McpTransportType::Stdio {
        if let Err(e) = validate_mcp_endpoint_url(&endpoint_url, &[]) {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": format!("Anti-SSRF: {}", e) }))).into_response();
        }
    }

    let raw_cred = payload.raw_credentials.as_deref().unwrap_or("");
    let auth_ref = match payload.auth_type {
        McpAuthType::Bearer => Some(("bearer", raw_cred)),
        McpAuthType::Basic => Some(("basic", raw_cred)),
        _ => None,
    };

    let http_client = McpHttpClient::new();
    let discovered = if payload.transport_type != McpTransportType::Stdio {
        match http_client.ping_and_discover(&endpoint_url, auth_ref).await {
            Ok(tools) => tools,
            Err(e) => return (StatusCode::BAD_GATEWAY, Json(json!({ "error": format!("Probe failed: {}", e) }))).into_response(),
        }
    } else {
        vec![]
    };

    let master_key = get_master_key();
    let encrypted_credentials = if !raw_cred.is_empty() {
        match encrypt_mcp_secret(raw_cred, &master_key) {
            Ok(enc) => Some(enc),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
        }
    } else {
        None
    };

    let server_id = format!("mcp-srv-{}", Uuid::new_v4());
    let now = Utc::now();
    let record = McpServerRecord {
        id: server_id.clone(),
        tenant_id,
        created_by: user_id,
        name: payload.name.trim().to_string(),
        display_title: payload.display_title.trim().to_string(),
        description: payload.description,
        transport_type: payload.transport_type,
        endpoint_url: payload.endpoint_url,
        command_bin: payload.command_bin,
        command_args: payload.command_args,
        auth_type: payload.auth_type,
        encrypted_credentials,
        is_enabled: true,
        risk_tier: payload.risk_tier.unwrap_or(McpRiskTier::Safe),
        scope_type: payload.scope_type.unwrap_or_else(|| "system".to_string()),
        owner_user_id: payload.owner_user_id,
        is_mandatory: payload.is_mandatory.unwrap_or(false),
        allowed_roles: payload.allowed_roles.unwrap_or_else(|| vec!["admin".to_string(), "user".to_string(), "auditor".to_string()]),
        oauth_config: payload.oauth_config,
        ssrf_policy: payload.ssrf_policy.unwrap_or_else(|| "strict".to_string()),
        egress_allowlist: payload.egress_allowlist,
        sandbox_type: payload.sandbox_type.unwrap_or_else(|| "restricted_process".to_string()),
        max_memory_mb: payload.max_memory_mb.unwrap_or(256),
        timeout_seconds: payload.timeout_seconds.unwrap_or(30),
        dlp_enabled: payload.dlp_enabled.unwrap_or(true),
        dlp_rules: payload.dlp_rules.unwrap_or_else(|| vec!["credit_card".to_string(), "nik".to_string(), "phone".to_string(), "email".to_string(), "jwt".to_string()]),
        rate_limit_rpm: payload.rate_limit_rpm.unwrap_or(60),
        daily_request_limit: payload.daily_request_limit.unwrap_or(1000),
        kms_provider: payload.kms_provider.unwrap_or_else(|| "local".to_string()),
        kms_key_id: payload.kms_key_id,
        created_at: now,
        updated_at: now,
    };

    if let Err(e) = state.repo.create_mcp_server(&record).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response();
    }

    let mut tool_records = Vec::with_capacity(discovered.len());
    for def in &discovered {
        tool_records.push(McpToolRecord {
            id: format!("mcp-tool-{}", Uuid::new_v4()),
            server_id: server_id.clone(),
            name: def.name.clone(),
            description: Some(def.description.clone()),
            input_schema_json: def.input_schema.to_string(),
            risk_tier: McpRiskTier::Safe,
            requires_approval: false,
            updated_at: now,
        });

        let proxy = Arc::new(RemoteMcpToolProxy::new(
            def.name.clone(),
            def.clone(),
            endpoint_url.clone(),
            auth_ref.map(|(s, t)| (s.to_string(), t.to_string())),
            http_client.clone(),
        ));
        let _ = state.mcp_registry.register_tool(proxy).await;
    }
    let _ = state.repo.sync_mcp_tools(&server_id, &tool_records).await;

    let mut resp = record;
    resp.encrypted_credentials = None;
    (StatusCode::CREATED, Json(json!({ "server": resp, "tools_count": discovered.len() }))).into_response()
}

/// GET /api/mcp/servers/{id}
pub async fn get_server(State(state): State<AppState>, Path(id): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    let (tenant_id, _, _) = extract_tenant_and_role(&headers, &state).await;
    match state.repo.get_mcp_server(&id, &tenant_id).await {
        Ok(Some(mut s)) => {
            s.encrypted_credentials = None;
            let tools = state.repo.list_mcp_tools(&s.id).await.unwrap_or_default();
            Json(json!({ "server": s, "tools": tools })).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// PUT /api/mcp/servers/{id}
pub async fn update_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<McpServerUpdateInput>,
) -> impl IntoResponse {
    let (tenant_id, _, role) = extract_tenant_and_role(&headers, &state).await;
    if !role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Forbidden" }))).into_response();
    }

    let existing = match state.repo.get_mcp_server(&id, &tenant_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Not found" }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    };

    let mut updated = existing;
    if let Some(t) = payload.display_title { updated.display_title = t; }
    if let Some(d) = payload.description { updated.description = Some(d); }
    if let Some(u) = payload.endpoint_url {
        if let Err(e) = validate_mcp_endpoint_url(&u, &[]) {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": format!("Anti-SSRF: {}", e) }))).into_response();
        }
        updated.endpoint_url = Some(u);
    }
    if let Some(raw) = payload.raw_credentials {
        if let Ok(enc) = encrypt_mcp_secret(&raw, &get_master_key()) {
            updated.encrypted_credentials = Some(enc);
        }
    }
    if let Some(en) = payload.is_enabled { updated.is_enabled = en; }
    if let Some(rt) = payload.risk_tier { updated.risk_tier = rt; }
    if let Some(m) = payload.is_mandatory { updated.is_mandatory = m; }
    if let Some(ar) = payload.allowed_roles { updated.allowed_roles = ar; }
    if let Some(oc) = payload.oauth_config { updated.oauth_config = Some(oc); }
    if let Some(sp) = payload.ssrf_policy { updated.ssrf_policy = sp; }
    if let Some(ea) = payload.egress_allowlist { updated.egress_allowlist = Some(ea); }
    if let Some(st) = payload.sandbox_type { updated.sandbox_type = st; }
    if let Some(mm) = payload.max_memory_mb { updated.max_memory_mb = mm; }
    if let Some(ts) = payload.timeout_seconds { updated.timeout_seconds = ts; }
    if let Some(de) = payload.dlp_enabled { updated.dlp_enabled = de; }
    if let Some(dr) = payload.dlp_rules { updated.dlp_rules = dr; }
    if let Some(rl) = payload.rate_limit_rpm { updated.rate_limit_rpm = rl; }
    if let Some(drl) = payload.daily_request_limit { updated.daily_request_limit = drl; }
    if let Some(kp) = payload.kms_provider { updated.kms_provider = kp; }
    if let Some(kid) = payload.kms_key_id { updated.kms_key_id = Some(kid); }
    updated.updated_at = Utc::now();

    match state.repo.update_mcp_server(&updated).await {
        Ok(mut res) => {
            res.encrypted_credentials = None;
            Json(json!({ "server": res })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// DELETE /api/mcp/servers/{id}
pub async fn delete_server(State(state): State<AppState>, Path(id): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    let (tenant_id, _, role) = extract_tenant_and_role(&headers, &state).await;
    if !role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Forbidden" }))).into_response();
    }

    if let Ok(Some(server)) = state.repo.get_mcp_server(&id, &tenant_id).await {
        if let Ok(tools) = state.repo.list_mcp_tools(&server.id).await {
            for t in tools {
                let _ = state.mcp_registry.unregister_tool(&t.name).await;
            }
        }
    }

    match state.repo.delete_mcp_server(&id, &tenant_id).await {
        Ok(true) => Json(json!({ "success": true })).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Server not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct TestProbeRequest {
    pub endpoint_url: String,
    pub auth_type: Option<McpAuthType>,
    pub raw_credentials: Option<String>,
}

/// POST /api/mcp/servers/test-endpoint
pub async fn test_endpoint(
    State(_state): State<AppState>,
    Json(payload): Json<TestProbeRequest>,
) -> impl IntoResponse {
    if let Err(e) = validate_mcp_endpoint_url(&payload.endpoint_url, &[]) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": format!("Anti-SSRF: {}", e) }))).into_response();
    }

    let auth_ref = match payload.auth_type.unwrap_or(McpAuthType::None) {
        McpAuthType::Bearer => payload.raw_credentials.as_deref().map(|t| ("bearer", t)),
        McpAuthType::Basic => payload.raw_credentials.as_deref().map(|t| ("basic", t)),
        _ => None,
    };

    let client = McpHttpClient::new();
    match client.ping_and_discover(&payload.endpoint_url, auth_ref).await {
        Ok(tools) => Json(json!({ "success": true, "tools": tools, "count": tools.len() })).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}
