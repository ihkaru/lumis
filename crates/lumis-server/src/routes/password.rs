#![deny(unsafe_code)]

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::User;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: Option<String>,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    let (username, _) = crate::routes::chat::helpers::resolve_claims_and_role(&state, &headers).await;
    if username == "anonymous" || username == "guest" || username.is_empty() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Autentikasi diperlukan" })),
        )
            .into_response();
    }

    let clean_new = payload.new_password.trim();
    if clean_new.len() < 4 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Kata sandi baru minimal 4 karakter" })),
        )
            .into_response();
    }

    match state.repo.get_user_by_username(&username).await {
        Ok(Some(mut user)) => {
            if !user.is_active {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({ "error": "Akun Anda dinonaktifkan" })),
                )
                    .into_response();
            }
            if user.password_hash.is_some() {
                let old_pass = payload.old_password.as_deref().unwrap_or("").trim();
                if !user.verify_password(old_pass) {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({ "error": "Kata sandi lama tidak sesuai" })),
                    )
                        .into_response();
                }
            }
            let now = chrono::Utc::now();
            user.password_hash = Some(User::hash_password(clean_new));
            user.updated_at = now;
            match state.repo.upsert_user(&user).await {
                Ok(_) => {
                    let identity = lumis_core::models::UserIdentity {
                        id: format!("idty-local-{}", user.id),
                        user_id: user.id.clone(),
                        provider: "local".to_string(),
                        provider_subject_id: user.username.clone(),
                        credential_data: user.password_hash.as_ref().map(|h| serde_json::json!({ "password_hash": h })),
                        last_login_at: Some(now),
                        created_at: user.created_at,
                        updated_at: now,
                    };
                    let _ = state.repo.link_identity(&identity).await;
                    (StatusCode::OK, Json(serde_json::json!({ "status": "password_updated" }))).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "User tidak ditemukan" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
