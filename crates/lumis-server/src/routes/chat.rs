use std::collections::HashMap;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use lumis_core::models::{
    CursorQuery, EnterpriseRole, MessageRole, NewMessage, NewSession, User,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

fn resolve_claims_and_role(state: &AppState, headers: &HeaderMap) -> (String, EnterpriseRole) {
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
        let role = state.claim_translator.resolve_effective_role(&claims);
        (claims.username, role)
    } else {
        ("default_user".to_string(), EnterpriseRole::Member)
    }
}

async fn ensure_user(state: &AppState, username: &str, role: EnterpriseRole) -> Result<User, String> {
    match state.repo.get_user_by_username(username).await {
        Ok(Some(u)) => Ok(u),
        Ok(None) => {
            let now = Utc::now();
            let new_user = User {
                id: Uuid::new_v4().to_string(),
                username: username.to_string(),
                email: None,
                role,
                tenant_id: None,
                preferences: serde_json::json!({}),
                is_active: true,
                created_at: now,
                updated_at: now,
            };
            state.repo.upsert_user(&new_user).await.map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn list_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationQuery>,
) -> impl IntoResponse {
    let (username, role) = resolve_claims_and_role(&state, &headers);
    let user_id = match ensure_user(&state, &username, role).await {
        Ok(u) => u.id,
        Err(_) => username,
    };

    let query = CursorQuery {
        cursor_time: params.cursor_time,
        cursor_id: params.cursor_id,
        limit: params.limit.unwrap_or(20),
    };

    match state.repo.list_sessions(&user_id, &query).await {
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
    headers: HeaderMap,
    Json(payload): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    let (username, role) = resolve_claims_and_role(&state, &headers);

    // Policy Enforcement Point (PEP): Validate Model Clearance
    let clearance_res = state.claim_translator.evaluate_model_clearance(role, &payload.model_id);
    let clearance_tier = match clearance_res {
        Ok(tier) => tier,
        Err(e) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: format!("Policy Enforcement Denied: {}", e),
                }),
            )
                .into_response();
        }
    };

    // JIT provision user to satisfy foreign key reference
    let user = match ensure_user(&state, &username, role).await {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
                .into_response();
        }
    };

    let new_session = NewSession {
        user_id: user.id,
        title: payload.title,
        model_id: payload.model_id,
        system_prompt: payload.system_prompt,
    };

    match state.repo.create_session(&new_session).await {
        Ok(session) => {
            let mut res = (StatusCode::CREATED, Json(session)).into_response();
            if let Ok(val) = HeaderValue::from_str(role.as_str()) {
                res.headers_mut().insert("x-forwarded-role", val);
            }
            if let Ok(val) = HeaderValue::from_str(clearance_tier.as_str()) {
                res.headers_mut().insert("x-forwarded-tier", val);
            }
            res
        }
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
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Json(payload): Json<SendMessageRequest>,
) -> impl IntoResponse {
    let (_username, role) = resolve_claims_and_role(&state, &headers);

    // 1. Policy Enforcement Point (PEP): Model Clearance verification
    if let Some(ref model_id) = payload.model_id {
        if let Err(e) = state.claim_translator.evaluate_model_clearance(role, model_id) {
            return (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: format!("Policy Enforcement Denied: {}", e),
                }),
            )
                .into_response();
        }
    }

    // 2. Guardrail inspection using SIMD Aho-Corasick
    if let Some(violation) = state.guardrail.check_violation(&payload.content) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Guardrail violation: restricted pattern '{}' detected", violation),
            }),
        )
            .into_response();
    }

    // 3. Append User Message
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

    // 4. Broadcast to zero-copy streaming channel
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
