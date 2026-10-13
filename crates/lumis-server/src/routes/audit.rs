#![deny(unsafe_code)]

use std::collections::HashMap;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::EnterpriseRole;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AuditQueryParam {
    pub format: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct KillswitchRequest {
    pub active: bool,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct KillswitchResponse {
    pub active: bool,
    pub reason: Option<String>,
    pub activated_by: Option<String>,
}

async fn resolve_identity(state: &AppState, headers: &HeaderMap) -> (String, EnterpriseRole, String) {
    let mut header_map = HashMap::new();
    for (k, v) in headers.iter() {
        if let Ok(str_val) = v.to_str() {
            header_map.insert(k.as_str().to_lowercase(), str_val.to_string());
        }
    }
    if let Some(val) = headers.get(&state.trusted_header).and_then(|v| v.to_str().ok()) {
        header_map.insert("x-forwarded-user".to_string(), val.to_string());
    }

    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default")
        .to_string();

    let (user_id, role) = if let Ok(claims) = state.claim_translator.extract_claims(&header_map) {
        if !claims.roles.is_empty() {
            let r = state.claim_translator.resolve_effective_role(&claims);
            (claims.username, r)
        } else {
            let r = match state.repo.get_user_by_username(&claims.username).await {
                Ok(Some(u)) => u.role,
                _ => EnterpriseRole::Member,
            };
            (claims.username, r)
        }
    } else {
        ("anonymous".to_string(), EnterpriseRole::Guest)
    };

    (tenant_id, role, user_id)
}

/// GET /api/admin/audit/events
/// Retrieves SIEM audit events. Restricted to TenantAdmin and SuperAdmin.
pub async fn get_audit_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(param): Query<AuditQueryParam>,
) -> impl IntoResponse {
    let (tenant_id, caller_role, _) = resolve_identity(&state, &headers).await;
    let limit = param.limit.unwrap_or(50);

    match state
        .audit_logger
        .query_recent_events(&tenant_id, caller_role, limit)
        .await
    {
        Ok(events) => {
            let format_type = param.format.as_deref().unwrap_or("raw");
            match format_type {
                "ecs" => {
                    let ecs_events: Vec<serde_json::Value> =
                        events.iter().map(|e| e.to_ecs_json()).collect();
                    (StatusCode::OK, Json(json!(ecs_events))).into_response()
                }
                "cef" => {
                    let cef_events: Vec<String> =
                        events.iter().map(|e| e.to_cef_string()).collect();
                    (StatusCode::OK, Json(json!(cef_events))).into_response()
                }
                _ => (StatusCode::OK, Json(json!(events))).into_response(),
            }
        }
        Err(lumis_core::error::LumisError::Forbidden(msg)) => {
            (StatusCode::FORBIDDEN, Json(json!({ "error": msg }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// GET /api/admin/emergency/killswitch
/// Returns current SOC emergency killswitch state.
pub async fn get_killswitch(State(state): State<AppState>) -> impl IntoResponse {
    let (active, reason, activated_by) = state.audit_logger.get_killswitch_status();
    (
        StatusCode::OK,
        Json(KillswitchResponse {
            active,
            reason,
            activated_by,
        }),
    )
        .into_response()
}

/// POST /api/admin/emergency/killswitch
/// Toggles the emergency killswitch. Restricted to TenantAdmin and SuperAdmin.
pub async fn set_killswitch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<KillswitchRequest>,
) -> impl IntoResponse {
    let (_, caller_role, user_id) = resolve_identity(&state, &headers).await;

    if !caller_role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only TenantAdmin or SuperAdmin can toggle emergency killswitch" })),
        )
            .into_response();
    }

    match state
        .audit_logger
        .set_killswitch(payload.active, &payload.reason, &user_id)
        .await
    {
        Ok(_) => {
            let (active, reason, activated_by) = state.audit_logger.get_killswitch_status();
            (
                StatusCode::OK,
                Json(KillswitchResponse {
                    active,
                    reason,
                    activated_by,
                }),
            )
                .into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}
