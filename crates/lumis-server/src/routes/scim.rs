#![deny(unsafe_code)]

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::scim::{ScimErrorResponse, ScimGroup, ScimListResponse, ScimUser};
use serde::Deserialize;

use crate::state::AppState;

/// GET /scim/v2/Users
pub async fn list_scim_users(State(state): State<AppState>) -> impl IntoResponse {
    match state.scim_provisioner.list_users().await {
        Ok(users) => (StatusCode::OK, Json(ScimListResponse::new(users))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ScimErrorResponse::new(500, e.to_string())),
        )
            .into_response(),
    }
}

/// GET /scim/v2/Users/{id}
pub async fn get_scim_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.scim_provisioner.get_user(&id).await {
        Ok(Some(u)) => (StatusCode::OK, Json(u)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ScimErrorResponse::new(404, format!("User '{}' not found", id))),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ScimErrorResponse::new(500, e.to_string())),
        )
            .into_response(),
    }
}

/// POST /scim/v2/Users
pub async fn create_scim_user(
    State(state): State<AppState>,
    Json(payload): Json<ScimUser>,
) -> impl IntoResponse {
    match state.scim_provisioner.create_user(payload).await {
        Ok(u) => (StatusCode::CREATED, Json(u)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ScimErrorResponse::new(400, e.to_string())),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ScimPatchOp {
    pub op: Option<String>,
    pub path: Option<String>,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PatchUserStatusRequest {
    pub active: Option<bool>,
    #[serde(rename = "Operations")]
    pub operations: Option<Vec<ScimPatchOp>>,
}

/// PATCH /scim/v2/Users/{id}
pub async fn patch_scim_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<PatchUserStatusRequest>,
) -> impl IntoResponse {
    let target_active = payload.active.or_else(|| {
        payload.operations.as_ref().and_then(|ops| {
            for op in ops {
                if let Some(path) = &op.path {
                    if path.eq_ignore_ascii_case("active") {
                        if let Some(val) = &op.value {
                            if let Some(b) = val.as_bool() {
                                return Some(b);
                            }
                            if let Some(s) = val.as_str() {
                                if s.eq_ignore_ascii_case("true") { return Some(true); }
                                if s.eq_ignore_ascii_case("false") { return Some(false); }
                            }
                        }
                    }
                } else if let Some(serde_json::Value::Object(map)) = &op.value {
                    if let Some(b) = map.get("active").and_then(|v| v.as_bool()) {
                        return Some(b);
                    }
                }
            }
            None
        })
    });

    if let Some(active) = target_active {
        match state.scim_provisioner.update_user_status(&id, active).await {
            Ok(true) => (StatusCode::NO_CONTENT).into_response(),
            Ok(false) => (
                StatusCode::NOT_FOUND,
                Json(ScimErrorResponse::new(404, format!("User '{}' not found", id))),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ScimErrorResponse::new(500, e.to_string())),
            )
                .into_response(),
        }
    } else {
        (StatusCode::NO_CONTENT).into_response()
    }
}

/// DELETE /scim/v2/Users/{id}
pub async fn delete_scim_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.scim_provisioner.delete_user(&id).await {
        Ok(true) => (StatusCode::NO_CONTENT).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ScimErrorResponse::new(404, format!("User '{}' not found", id))),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ScimErrorResponse::new(500, e.to_string())),
        )
            .into_response(),
    }
}

/// GET /scim/v2/Groups
pub async fn list_scim_groups(State(state): State<AppState>) -> impl IntoResponse {
    match state.scim_provisioner.list_groups().await {
        Ok(groups) => (StatusCode::OK, Json(ScimListResponse::new(groups))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ScimErrorResponse::new(500, e.to_string())),
        )
            .into_response(),
    }
}

/// GET /scim/v2/Groups/{id}
pub async fn get_scim_group(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.scim_provisioner.get_group(&id).await {
        Ok(Some(g)) => (StatusCode::OK, Json(g)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ScimErrorResponse::new(404, format!("Group '{}' not found", id))),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ScimErrorResponse::new(500, e.to_string())),
        )
            .into_response(),
    }
}

/// POST /scim/v2/Groups
pub async fn create_scim_group(
    State(state): State<AppState>,
    Json(payload): Json<ScimGroup>,
) -> impl IntoResponse {
    match state.scim_provisioner.create_group(payload).await {
        Ok(g) => (StatusCode::CREATED, Json(g)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ScimErrorResponse::new(400, e.to_string())),
        )
            .into_response(),
    }
}

/// DELETE /scim/v2/Groups/{id}
pub async fn delete_scim_group(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.scim_provisioner.delete_group(&id).await {
        Ok(true) => (StatusCode::NO_CONTENT).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ScimErrorResponse::new(404, format!("Group '{}' not found", id))),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ScimErrorResponse::new(500, e.to_string())),
        )
            .into_response(),
    }
}
