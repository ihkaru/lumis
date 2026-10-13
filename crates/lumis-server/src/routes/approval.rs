#![deny(unsafe_code)]

use std::collections::HashMap;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::audit::{AuditEventStatus, AuditLogEvent};
use lumis_core::models::EnterpriseRole;
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

async fn resolve_role_and_user(state: &AppState, headers: &HeaderMap) -> (String, EnterpriseRole) {
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
        if !claims.roles.is_empty() {
            let role = state.claim_translator.resolve_effective_role(&claims);
            (claims.username, role)
        } else {
            let r = match state.repo.get_user_by_username(&claims.username).await {
                Ok(Some(u)) => u.role,
                _ => EnterpriseRole::Member,
            };
            (claims.username, r)
        }
    } else {
        ("guest".to_string(), EnterpriseRole::Guest)
    }
}

/// GET /api/mcp/approvals/pending
/// Lists pending tool execution approvals (Requires TenantAdmin+).
pub async fn list_pending_approvals(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_, role) = resolve_role_and_user(&state, &headers).await;
    if !role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Insufficient privileges to review approvals" }))).into_response();
    }

    match state.approval_manager.list_pending_approvals().await {
        Ok(list) => (StatusCode::OK, Json(list)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// GET /api/mcp/approvals/{token}
pub async fn get_approval(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    match state.approval_manager.get_approval(&token).await {
        Ok(Some(appr)) => (StatusCode::OK, Json(appr)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Approval request not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct DenyApprovalRequest {
    pub reason: Option<String>,
}

/// POST /api/mcp/approvals/{token}/approve
pub async fn approve_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let (username, role) = resolve_role_and_user(&state, &headers).await;
    if !role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Approval requires TenantAdmin clearance" }))).into_response();
    }

    match state.approval_manager.approve(&token, &username).await {
        Ok(appr) => {
            let audit_event = AuditLogEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                tenant_id: "default".to_string(),
                actor_id: username.clone(),
                actor_role: role,
                action: "tool_approval_granted".to_string(),
                resource_id: Some(appr.token.clone()),
                status: AuditEventStatus::Success,
                ip_address: None,
                user_agent: None,
                details: json!({ "tool_name": appr.tool_name }),
            };
            let _ = state.audit_logger.log_event(&audit_event).await;
            (StatusCode::OK, Json(appr)).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// POST /api/mcp/approvals/{token}/deny
pub async fn deny_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token): Path<String>,
    Json(payload): Json<DenyApprovalRequest>,
) -> impl IntoResponse {
    let (username, role) = resolve_role_and_user(&state, &headers).await;
    if !role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Denying approvals requires TenantAdmin clearance" }))).into_response();
    }

    match state.approval_manager.deny(&token, &username, payload.reason.as_deref()).await {
        Ok(appr) => {
            let audit_event = AuditLogEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                tenant_id: "default".to_string(),
                actor_id: username.clone(),
                actor_role: role,
                action: "tool_approval_denied".to_string(),
                resource_id: Some(appr.token.clone()),
                status: AuditEventStatus::Failure,
                ip_address: None,
                user_agent: None,
                details: json!({ "tool_name": appr.tool_name, "reason": payload.reason }),
            };
            let _ = state.audit_logger.log_event(&audit_event).await;
            (StatusCode::OK, Json(appr)).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}
