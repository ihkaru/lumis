use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::{CursorQuery, NewSession};

use crate::state::AppState;
use super::helpers::{ensure_user, resolve_claims_and_role};
use super::types::{CreateSessionRequest, ErrorResponse, PaginationQuery};

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
        id: None,
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
