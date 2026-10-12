use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::{EnterpriseRole, User};
use serde::Serialize;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub authenticated: bool,
    pub user: Option<User>,
    pub role: EnterpriseRole,
}

pub async fn auth_me(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 1. Extract trusted identity from forward auth header
    let username = headers
        .get(&state.trusted_header)
        .and_then(|h| h.to_str().ok())
        .or_else(|| {
            headers
                .get("x-forwarded-user")
                .and_then(|h| h.to_str().ok())
        })
        .unwrap_or("anonymous");

    if username == "anonymous" {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "authenticated": false, "error": "Missing SSO session" })),
        )
            .into_response();
    }

    // 2. Resolve or JIT-provision user
    match state.repo.get_user_by_username(username).await {
        Ok(Some(existing_user)) => (
            StatusCode::OK,
            Json(AuthResponse {
                authenticated: true,
                role: existing_user.role,
                user: Some(existing_user),
            }),
        )
            .into_response(),
        Ok(None) => {
            // JIT Provisioning
            let now = chrono::Utc::now();
            let new_user = User {
                id: Uuid::new_v4().to_string(),
                username: username.to_string(),
                email: None,
                role: EnterpriseRole::Member,
                tenant_id: None,
                preferences: serde_json::json!({}),
                is_active: true,
                created_at: now,
                updated_at: now,
            };

            match state.repo.upsert_user(&new_user).await {
                Ok(created) => (
                    StatusCode::OK,
                    Json(AuthResponse {
                        authenticated: true,
                        role: created.role,
                        user: Some(created),
                    }),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
