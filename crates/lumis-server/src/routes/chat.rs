use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use lumis_core::models::{CursorQuery, MessageRole, NewMessage, NewSession};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub cursor_time: Option<DateTime<Utc>>,
    pub cursor_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: String,
    pub model_id: String,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub model_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Query(params): Query<PaginationQuery>,
) -> impl IntoResponse {
    let user_id = "default_user"; // In production, resolved from SSO claims
    let query = CursorQuery {
        cursor_time: params.cursor_time,
        cursor_id: params.cursor_id,
        limit: params.limit.unwrap_or(20),
    };

    match state.repo.list_sessions(user_id, &query).await {
        Ok(page) => (StatusCode::OK, Json(page)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    let user_id = "default_user".to_string();
    let new_session = NewSession {
        user_id,
        title: payload.title,
        model_id: payload.model_id,
        system_prompt: payload.system_prompt,
    };

    match state.repo.create_session(&new_session).await {
        Ok(session) => (StatusCode::CREATED, Json(session)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn list_messages(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<PaginationQuery>,
) -> impl IntoResponse {
    let query = CursorQuery {
        cursor_time: params.cursor_time,
        cursor_id: params.cursor_id,
        limit: params.limit.unwrap_or(20),
    };

    match state.repo.list_messages(&session_id, &query).await {
        Ok(page) => (StatusCode::OK, Json(page)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn send_message(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(payload): Json<SendMessageRequest>,
) -> impl IntoResponse {
    // 1. Guardrail inspection using SIMD Aho-Corasick
    if let Some(violation) = state.guardrail.check_violation(&payload.content) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Guardrail violation: restricted pattern '{}' detected", violation),
            }),
        )
            .into_response();
    }

    // 2. Append User Message
    let user_msg = NewMessage {
        session_id: session_id.clone(),
        role: MessageRole::User,
        content: payload.content.clone(),
        model_id: payload.model_id.clone(),
        token_count: None,
        metadata: None,
    };

    if let Err(e) = state.repo.append_message(&user_msg).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response();
    }

    // 3. Broadcast to zero-copy streaming channel
    let event_payload = serde_json::json!({
        "event": "user_message",
        "session_id": session_id,
        "content": payload.content,
    });
    let _ = state
        .stream_tx
        .send(bytes::Bytes::from(event_payload.to_string()));

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "received",
            "session_id": session_id,
        })),
    )
        .into_response()
}
