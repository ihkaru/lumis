#![deny(unsafe_code)]

use std::collections::HashMap;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use lumis_core::models::mcp::{
    McpRiskTier, McpServerCreateInput, McpServerRecord,
    McpTransportType, UserMcpOAuthTokenRecord,
};
use lumis_core::models::EnterpriseRole;
use lumis_core::oauth::{build_mcp_authorization_url, generate_pkce_pair};
use lumis_core::security::{encrypt_mcp_secret, validate_mcp_endpoint_url};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::state::AppState;

fn get_master_key() -> Vec<u8> {
    std::env::var("LUMIS_MCP_ENCRYPTION_KEY")
        .or_else(|_| std::env::var("LUMIS_SECRET_KEY"))
        .unwrap_or_else(|_| "lumis-default-master-key-32b-secret!".to_string())
        .into_bytes()
}

async fn extract_tenant_and_user(headers: &HeaderMap, state: &AppState) -> (String, String, EnterpriseRole) {
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

#[derive(Deserialize)]
pub struct OAuthAuthorizeRequest {
    pub redirect_uri: String,
    pub state: Option<String>,
}

/// POST /api/mcp/servers/{id}/oauth/authorize
pub async fn authorize_mcp_oauth(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<OAuthAuthorizeRequest>,
) -> impl IntoResponse {
    let (tenant_id, _, _) = extract_tenant_and_user(&headers, &state).await;
    let server = match state.repo.get_mcp_server(&id, &tenant_id).await {
        Ok(Some(s)) => s,
        _ => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Server not found" }))).into_response(),
    };

    let oauth_cfg = match server.oauth_config {
        Some(cfg) => cfg,
        None => return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Server does not configure OAuth 2.1" }))).into_response(),
    };

    let pkce = generate_pkce_pair();
    let state_param = payload.state.unwrap_or_else(|| Uuid::new_v4().to_string());
    let resource = oauth_cfg.resource_indicator.as_deref().or(server.endpoint_url.as_deref());

    let auth_url = build_mcp_authorization_url(
        &oauth_cfg.authorization_endpoint,
        &oauth_cfg.client_id,
        &payload.redirect_uri,
        &pkce.code_challenge,
        &oauth_cfg.scopes,
        &state_param,
        resource,
    );

    Json(json!({
        "authorization_url": auth_url,
        "state": state_param,
        "code_verifier": pkce.code_verifier,
        "code_challenge_method": pkce.code_challenge_method,
    })).into_response()
}

#[derive(Deserialize)]
pub struct OAuthCallbackRequest {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub expires_in: Option<i64>,
}

/// POST /api/mcp/servers/{id}/oauth/callback
pub async fn callback_mcp_oauth(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<OAuthCallbackRequest>,
) -> impl IntoResponse {
    let (tenant_id, user_id, _) = extract_tenant_and_user(&headers, &state).await;
    if user_id == "anonymous" {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Authentication required" }))).into_response();
    }

    let server = match state.repo.get_mcp_server(&id, &tenant_id).await {
        Ok(Some(s)) => s,
        _ => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Server not found" }))).into_response(),
    };

    let master_key = get_master_key();
    let enc_access = match encrypt_mcp_secret(&payload.access_token, &master_key) {
        Ok(enc) => enc,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    };

    let enc_refresh = if let Some(ref r) = payload.refresh_token {
        encrypt_mcp_secret(r, &master_key).ok()
    } else {
        None
    };

    let now = Utc::now();
    let expires_at = payload.expires_in.map(|secs| now.timestamp() + secs);
    let token_record = UserMcpOAuthTokenRecord {
        id: format!("mcp-tok-{}", Uuid::new_v4()),
        user_id,
        server_id: server.id,
        access_token_encrypted: enc_access,
        refresh_token_encrypted: enc_refresh,
        token_type: payload.token_type.unwrap_or_else(|| "Bearer".to_string()),
        scope: payload.scope,
        expires_at,
        created_at: now,
        updated_at: now,
    };

    match state.repo.save_user_mcp_oauth_token(&token_record).await {
        Ok(_) => Json(json!({ "success": true, "connected": true })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// DELETE /api/mcp/servers/{id}/oauth/token
pub async fn revoke_mcp_oauth_token(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_, user_id, _) = extract_tenant_and_user(&headers, &state).await;
    match state.repo.delete_user_mcp_oauth_token(&user_id, &id).await {
        Ok(_) => Json(json!({ "success": true, "disconnected": true })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Serialize)]
pub struct UserMcpServerStatus {
    pub id: String,
    pub name: String,
    pub display_title: String,
    pub description: Option<String>,
    pub transport_type: String,
    pub auth_type: String,
    pub scope_type: String,
    pub is_mandatory: bool,
    pub is_enabled: bool,
    pub is_oauth_connected: bool,
    pub tools_count: usize,
}

/// GET /api/mcp/preferences
pub async fn list_user_mcp_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (tenant_id, user_id, role) = extract_tenant_and_user(&headers, &state).await;
    let servers = state.repo.list_mcp_servers_for_user(&tenant_id, &user_id, role.as_str()).await.unwrap_or_default();
    let preferences = state.repo.list_user_mcp_preferences(&user_id).await.unwrap_or_default();

    let mut pref_map = HashMap::new();
    for p in preferences {
        pref_map.insert(p.server_id, p.is_enabled);
    }

    let mut status_list = Vec::with_capacity(servers.len());
    for s in servers {
        let is_oauth_connected = state.repo.get_user_mcp_oauth_token(&user_id, &s.id).await.ok().flatten().is_some();
        let tools = state.repo.list_mcp_tools(&s.id).await.unwrap_or_default();
        
        let is_enabled = if s.is_mandatory {
            true // CISO Enforced: selalu aktif untuk server mandatory
        } else {
            pref_map.get(&s.id).copied().unwrap_or(s.is_enabled)
        };

        status_list.push(UserMcpServerStatus {
            id: s.id,
            name: s.name,
            display_title: s.display_title,
            description: s.description,
            transport_type: s.transport_type.as_str().to_string(),
            auth_type: s.auth_type.as_str().to_string(),
            scope_type: s.scope_type,
            is_mandatory: s.is_mandatory,
            is_enabled,
            is_oauth_connected,
            tools_count: tools.len(),
        });
    }

    Json(json!({ "servers": status_list })).into_response()
}

#[derive(Deserialize)]
pub struct TogglePreferenceRequest {
    pub is_enabled: bool,
}

/// PUT /api/mcp/preferences/{server_id}
pub async fn toggle_user_mcp_preference(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<TogglePreferenceRequest>,
) -> impl IntoResponse {
    let (tenant_id, user_id, _) = extract_tenant_and_user(&headers, &state).await;
    if let Ok(Some(server)) = state.repo.get_mcp_server(&server_id, &tenant_id).await {
        // CISO Enforcement Rule: Server mandatory DILARANG dimatikan
        if server.is_mandatory && !payload.is_enabled {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Server ini bersifat Mandatory (CISO Enforced) dan tidak dapat dinonaktifkan." })),
            ).into_response();
        }
    }

    match state.repo.set_user_mcp_preference(&user_id, &server_id, payload.is_enabled).await {
        Ok(_) => Json(json!({ "success": true, "server_id": server_id, "is_enabled": payload.is_enabled })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// POST /api/mcp/servers/user
pub async fn create_user_mcp_server(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<McpServerCreateInput>,
) -> impl IntoResponse {
    let (tenant_id, user_id, _) = extract_tenant_and_user(&headers, &state).await;
    if user_id == "anonymous" {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Authentication required" }))).into_response();
    }

    let endpoint_url = payload.endpoint_url.clone().unwrap_or_default();
    if payload.transport_type != McpTransportType::Stdio {
        if let Err(e) = validate_mcp_endpoint_url(&endpoint_url, &[]) {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": format!("Anti-SSRF: {}", e) }))).into_response();
        }
    }

    let server_id = format!("mcp-usr-{}", Uuid::new_v4());
    let now = Utc::now();
    let record = McpServerRecord {
        id: server_id.clone(),
        tenant_id,
        created_by: user_id.clone(),
        name: payload.name.trim().to_string(),
        display_title: payload.display_title.trim().to_string(),
        description: payload.description,
        transport_type: payload.transport_type,
        endpoint_url: payload.endpoint_url,
        command_bin: payload.command_bin,
        command_args: payload.command_args,
        auth_type: payload.auth_type,
        encrypted_credentials: None,
        is_enabled: true,
        risk_tier: payload.risk_tier.unwrap_or(McpRiskTier::Safe),
        scope_type: "user".to_string(),
        owner_user_id: Some(user_id),
        is_mandatory: false,
        allowed_roles: vec!["admin".to_string(), "user".to_string(), "member".to_string()],
        oauth_config: payload.oauth_config,
        created_at: now,
        updated_at: now,
    };

    match state.repo.create_mcp_server(&record).await {
        Ok(res) => (StatusCode::CREATED, Json(json!({ "server": res }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// DELETE /api/mcp/servers/user/{id}
pub async fn delete_user_mcp_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (tenant_id, user_id, role) = extract_tenant_and_user(&headers, &state).await;
    let existing = match state.repo.get_mcp_server(&id, &tenant_id).await {
        Ok(Some(s)) => s,
        _ => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Server not found" }))).into_response(),
    };

    // User hanya boleh menghapus server miliknya sendiri, kecuali Admin
    if existing.owner_user_id.as_deref() != Some(&user_id) && !role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Forbidden" }))).into_response();
    }

    match state.repo.delete_mcp_server(&id, &tenant_id).await {
        Ok(true) => Json(json!({ "success": true })).into_response(),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to delete" }))).into_response(),
    }
}
