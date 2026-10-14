use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use bytes::Bytes;
use lumis_core::models::{CreateTaskInput, UpdateTaskInput};
use serde_json::json;

use crate::state::AppState;
use super::types::ErrorResponse;

pub async fn list_session_tasks(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.repo.list_session_tasks(&session_id).await {
        Ok(tasks) => (StatusCode::OK, Json(tasks)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response(),
    }
}

pub async fn create_session_task(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(mut payload): Json<CreateTaskInput>,
) -> impl IntoResponse {
    payload.session_id = session_id.clone();
    match state.repo.create_session_task(&payload).await {
        Ok(task) => {
            let evt = json!({
                "event": "task_created",
                "session_id": session_id,
                "task": task,
            });
            let _ = state.stream_tx.send(Bytes::from(evt.to_string()));
            (StatusCode::CREATED, Json(task)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response(),
    }
}

pub async fn update_session_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(payload): Json<UpdateTaskInput>,
) -> impl IntoResponse {
    match state.repo.update_session_task(&task_id, &payload).await {
        Ok(Some(task)) => {
            let evt = json!({
                "event": "task_updated",
                "session_id": task.session_id,
                "task": task,
            });
            let _ = state.stream_tx.send(Bytes::from(evt.to_string()));
            (StatusCode::OK, Json(task)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: format!("Task '{}' not found", task_id) }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response(),
    }
}

pub async fn delete_session_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    // Find task first to know session_id for event emission
    let existing = state.repo.get_session_task(&task_id).await.ok().flatten();
    match state.repo.delete_session_task(&task_id).await {
        Ok(true) => {
            if let Some(task) = existing {
                let evt = json!({
                    "event": "task_deleted",
                    "session_id": task.session_id,
                    "task_id": task_id,
                });
                let _ = state.stream_tx.send(Bytes::from(evt.to_string()));
            }
            (StatusCode::OK, Json(json!({ "deleted": true, "task_id": task_id }))).into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: format!("Task '{}' not found", task_id) }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response(),
    }
}
