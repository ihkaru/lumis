use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::NewUserMemory;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use super::chat::helpers::{ensure_user, resolve_claims_and_role};

#[derive(Debug, Deserialize)]
pub struct CreateMemoryRequest {
    pub category: Option<String>,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryResponse<T> {
    pub data: T,
}

pub async fn list_memories(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (username, role) = resolve_claims_and_role(&state, &headers).await;
    let user = match ensure_user(&state, &username, role).await {
        Ok(u) => u,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))).into_response(),
    };

    match state.repo.list_user_memories(&user.id).await {
        Ok(memories) => (StatusCode::OK, Json(serde_json::json!({ "memories": memories }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn create_memory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateMemoryRequest>,
) -> impl IntoResponse {
    let (username, role) = resolve_claims_and_role(&state, &headers).await;
    let user = match ensure_user(&state, &username, role).await {
        Ok(u) => u,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))).into_response(),
    };

    if payload.content.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Content cannot be empty" }))).into_response();
    }

    let new_mem = NewUserMemory {
        user_id: user.id,
        category: payload.category,
        content: payload.content.trim().to_string(),
    };

    match state.repo.add_user_memory(&new_mem).await {
        Ok(mem) => (StatusCode::CREATED, Json(mem)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_memory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (username, role) = resolve_claims_and_role(&state, &headers).await;
    let user = match ensure_user(&state, &username, role).await {
        Ok(u) => u,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))).into_response(),
    };

    match state.repo.delete_user_memory(&id, &user.id).await {
        Ok(deleted) => {
            if deleted {
                (StatusCode::OK, Json(serde_json::json!({ "status": "deleted", "id": id }))).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Memory not found" }))).into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
