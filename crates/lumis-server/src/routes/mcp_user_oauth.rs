#![deny(unsafe_code)]

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use lumis_core::models::mcp::UserMcpOAuthTokenRecord;
use lumis_core::oauth::{build_mcp_authorization_url, generate_pkce_pair};
use lumis_core::security::encrypt_mcp_secret;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::state::AppState;

pub use super::mcp_user_servers::{
    create_user_mcp_server, delete_user_mcp_server, extract_tenant_and_user,
    list_user_mcp_preferences, toggle_user_mcp_preference,
};

fn get_master_key() -> Vec<u8> {
    std::env::var("LUMIS_MCP_ENCRYPTION_KEY")
        .or_else(|_| std::env::var("LUMIS_SECRET_KEY"))
        .unwrap_or_else(|_| "lumis-default-master-key-32b-secret!".to_string())
        .into_bytes()
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
        "code_verifier": pkce.code_verifier,
        "state": state_param,
    })).into_response()
}

#[derive(Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: Option<String>,
    pub code_verifier: Option<String>,
    pub redirect_uri: Option<String>,
    pub access_token: Option<String>,
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
    let server = match state.repo.get_mcp_server(&id, &tenant_id).await {
        Ok(Some(s)) => s,
        _ => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Server not found" }))).into_response(),
    };

    let oauth_cfg = match server.oauth_config {
        Some(cfg) => cfg,
        None => return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Server does not configure OAuth 2.1" }))).into_response(),
    };

    let (access_token, refresh_token, token_type, scope, expires_in) = if let Some(ref at) = payload.access_token {
        (
            at.clone(),
            payload.refresh_token.clone(),
            payload.token_type.clone().unwrap_or_else(|| "Bearer".to_string()),
            payload.scope.clone(),
            payload.expires_in,
        )
    } else if let (Some(code), Some(verifier), Some(redirect_uri)) = (&payload.code, &payload.code_verifier, &payload.redirect_uri) {
        let client = reqwest::Client::new();
        let mut params = std::collections::HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("client_id", &oauth_cfg.client_id);
        params.insert("code", code.as_str());
        params.insert("redirect_uri", redirect_uri.as_str());
        params.insert("code_verifier", verifier.as_str());

        let res = match client.post(&oauth_cfg.token_endpoint).form(&params).send().await {
            Ok(r) => r,
            Err(e) => return (StatusCode::BAD_GATEWAY, Json(json!({ "error": format!("Token request failed: {}", e) }))).into_response(),
        };

        if !res.status().is_success() {
            let err_body = res.text().await.unwrap_or_default();
            return (StatusCode::BAD_GATEWAY, Json(json!({ "error": format!("Token endpoint rejected: {}", err_body) }))).into_response();
        }

        let token_data: serde_json::Value = match res.json().await {
            Ok(j) => j,
            Err(e) => return (StatusCode::BAD_GATEWAY, Json(json!({ "error": format!("Invalid token response: {}", e) }))).into_response(),
        };

        let at = token_data.get("access_token").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let rt = token_data.get("refresh_token").and_then(|v| v.as_str()).map(|s| s.to_string());
        let tt = token_data.get("token_type").and_then(|v| v.as_str()).unwrap_or("Bearer").to_string();
        let sc = token_data.get("scope").and_then(|v| v.as_str()).map(|s| s.to_string());
        let exp = token_data.get("expires_in").and_then(|v| v.as_i64());
        (at, rt, tt, sc, exp)
    } else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Missing code or access_token in callback" }))).into_response();
    };

    if access_token.is_empty() {
        return (StatusCode::BAD_GATEWAY, Json(json!({ "error": "Empty access_token from IdP" }))).into_response();
    }

    let master_key = get_master_key();
    let access_encrypted = match encrypt_mcp_secret(&access_token, &master_key) {
        Ok(enc) => enc,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    };

    let refresh_encrypted = if let Some(ref rf) = refresh_token {
        encrypt_mcp_secret(rf, &master_key).ok()
    } else {
        None
    };

    let now = Utc::now();
    let expires_at = expires_in.map(|sec| now.timestamp() + sec);

    let token_record = UserMcpOAuthTokenRecord {
        id: format!("mcp-tok-{}", Uuid::new_v4()),
        user_id,
        server_id: server.id,
        access_token_encrypted: access_encrypted,
        refresh_token_encrypted: refresh_encrypted,
        token_type: token_type.to_string(),
        scope: scope.map(|s| s.to_string()),
        expires_at,
        is_revoked: false,
        revoked_at: None,
        idp_session_id: None,
        created_at: now,
        updated_at: now,
    };

    if let Err(e) = state.repo.save_user_mcp_oauth_token(&token_record).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response();
    }

    Json(json!({
        "success": true,
        "token_type": token_type,
        "expires_in": expires_in,
    })).into_response()
}

/// DELETE /api/mcp/servers/{id}/oauth/token
pub async fn revoke_mcp_oauth_token(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_, user_id, _) = extract_tenant_and_user(&headers, &state).await;
    match state.repo.delete_user_mcp_oauth_token(&user_id, &id).await {
        Ok(true) => Json(json!({ "success": true, "message": "OAuth token revoked" })).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Token not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct BackchannelLogoutRequest {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

/// POST /api/mcp/oauth/backchannel-logout (RFC 7009 / OIDC Backchannel Logout)
pub async fn backchannel_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<BackchannelLogoutRequest>,
) -> impl IntoResponse {
    let (_, req_user_id, role) = extract_tenant_and_user(&headers, &state).await;
    let target_user = payload.user_id.unwrap_or(req_user_id);

    // Only Admin can revoke tokens on behalf of another user
    if target_user != "anonymous" && !role.is_at_least(lumis_core::models::EnterpriseRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Forbidden" }))).into_response();
    }

    match state.repo.revoke_tokens_by_session_or_user(&target_user, payload.session_id.as_deref()).await {
        Ok(count) => Json(json!({ "success": true, "revoked_tokens_count": count })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}
