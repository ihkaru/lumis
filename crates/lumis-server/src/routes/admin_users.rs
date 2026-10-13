#![deny(unsafe_code)]

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::EnterpriseRole;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub username: String,
    pub role: String,
}

pub async fn list_admin_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_username, caller_role) = crate::routes::chat::helpers::resolve_claims_and_role(&state, &headers).await;
    if !caller_role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Only TenantAdmin or SuperAdmin can list enterprise users" })),
        )
            .into_response();
    }

    match state.repo.list_users(100).await {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn update_user_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateUserRoleRequest>,
) -> impl IntoResponse {
    let (_username, caller_role) = crate::routes::chat::helpers::resolve_claims_and_role(&state, &headers).await;
    if !caller_role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Only TenantAdmin or SuperAdmin can update user roles" })),
        )
            .into_response();
    }

    let target_role = EnterpriseRole::from_str(&payload.role);
    if target_role == EnterpriseRole::SuperAdmin && caller_role != EnterpriseRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Only SuperAdmin can promote users to SuperAdmin" })),
        )
            .into_response();
    }

    match state.repo.get_user_by_username(&payload.username).await {
        Ok(Some(mut user)) => {
            user.role = target_role;
            user.updated_at = chrono::Utc::now();
            match state.repo.upsert_user(&user).await {
                Ok(updated) => (StatusCode::OK, Json(updated)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Ok(None) => {
            let now = chrono::Utc::now();
            let new_u = lumis_core::models::User {
                id: uuid::Uuid::new_v4().to_string(),
                username: payload.username.clone(),
                email: None,
                role: target_role,
                tenant_id: None,
                preferences: serde_json::json!({}),
                is_active: true,
                password_hash: None,
                created_at: now,
                updated_at: now,
            };
            match state.repo.upsert_user(&new_u).await {
                Ok(u) => (StatusCode::OK, Json(u)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
