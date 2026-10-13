#![deny(unsafe_code)]

use std::collections::HashMap;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::{EnterpriseRole, McpToolCallRequest};
use serde_json::json;

use crate::state::AppState;

async fn resolve_role(state: &AppState, headers: &HeaderMap) -> EnterpriseRole {
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
            state.claim_translator.resolve_effective_role(&claims)
        } else {
            match state.repo.get_user_by_username(&claims.username).await {
                Ok(Some(u)) => u.role,
                _ => EnterpriseRole::Member,
            }
        }
    } else {
        EnterpriseRole::Guest
    }
}

/// GET /api/mcp/tools
/// Lists all tools accessible to the caller's role level.
/// Supports conditional caching with ETag and If-None-Match to eliminate repeated querying (LibreChat#15757).
pub async fn list_tools(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let role = resolve_role(&state, &headers).await;
    let current_etag = state.mcp_registry.catalog_etag();

    if let Some(if_none_match) = headers.get("if-none-match").and_then(|v| v.to_str().ok()) {
        if if_none_match.trim() == current_etag {
            return (
                StatusCode::NOT_MODIFIED,
                [("etag", current_etag)],
                Json(serde_json::Value::Null),
            ).into_response();
        }
    }

    let tools = state.mcp_registry.list_tools(role).await;
    (
        StatusCode::OK,
        [("etag", current_etag)],
        Json(serde_json::to_value(tools).unwrap_or(serde_json::Value::Null)),
    ).into_response()
}

/// POST /api/mcp/tools/call
/// Synchronously executes an MCP tool with EMA role-gated verification.
pub async fn call_tool(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<McpToolCallRequest>,
) -> impl IntoResponse {
    let role = resolve_role(&state, &headers).await;
    match state.mcp_registry.execute_tool(role, &payload).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(lumis_core::error::LumisError::Forbidden(msg)) => {
            (StatusCode::FORBIDDEN, Json(json!({ "error": msg }))).into_response()
        }
        Err(lumis_core::error::LumisError::NotFound(msg)) => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": msg }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// POST /api/mcp/tasks
/// Spawns an asynchronous task under the MCP 2026 Tasks Extension.
pub async fn spawn_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<McpToolCallRequest>,
) -> impl IntoResponse {
    let role = resolve_role(&state, &headers).await;
    match state.mcp_registry.spawn_task(role, &payload).await {
        Ok(status) => (StatusCode::ACCEPTED, Json(status)).into_response(),
        Err(lumis_core::error::LumisError::Forbidden(msg)) => {
            (StatusCode::FORBIDDEN, Json(json!({ "error": msg }))).into_response()
        }
        Err(lumis_core::error::LumisError::NotFound(msg)) => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": msg }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// GET /api/mcp/tasks/{task_id}
/// Retrieves the execution status and output of an async MCP task.
pub async fn get_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match state.mcp_registry.get_task_status(&task_id).await {
        Ok(Some(status)) => (StatusCode::OK, Json(status)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Task not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// POST /api/mcp/tasks/{task_id}/cancel
/// Cancels a running or queued asynchronous MCP task.
pub async fn cancel_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match state.mcp_registry.cancel_task(&task_id).await {
        Ok(true) => (StatusCode::OK, Json(json!({ "cancelled": true }))).into_response(),
        Ok(false) => (StatusCode::BAD_REQUEST, Json(json!({ "cancelled": false, "error": "Task cannot be cancelled" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}
