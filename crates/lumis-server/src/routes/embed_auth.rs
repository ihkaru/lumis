#![deny(unsafe_code)]

//! # Ephemeral Embed Token Authentication
//! Protocol: Phase 19.3 & RFC 6454 / OWASP Top 10
//! Issues and verifies HMAC-SHA256 signed embed tokens for cross-origin host SPA embedding.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateEmbedTokenRequest {
    pub external_user_id: String,
    pub username: Option<String>,
    pub tenant_id: Option<String>,
    pub roles: Option<Vec<String>>,
    pub email: Option<String>,
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreateEmbedTokenResponse {
    pub token: String,
    pub user_id: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmbedTokenResponse {
    pub valid: bool,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub tenant_id: Option<String>,
    pub roles: Vec<String>,
    pub expires_at: Option<i64>,
    pub error: Option<String>,
}

fn get_secret_key() -> String {
    std::env::var("EMBED_TOKEN_SECRET")
        .or_else(|_| std::env::var("APP_SECRET"))
        .unwrap_or_else(|_| "lumis-default-embed-secret-key-2026".to_string())
}

fn compute_signature(secret: &str, payload_b64: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b":lumis_embed:");
    hasher.update(payload_b64.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

pub async fn issue_embed_token(
    State(_state): State<AppState>,
    Json(payload): Json<CreateEmbedTokenRequest>,
) -> impl IntoResponse {
    if payload.external_user_id.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "external_user_id cannot be empty" })),
        )
            .into_response();
    }

    let ttl = payload.ttl_seconds.unwrap_or(3600).clamp(60, 86400);
    let expires_at = Utc::now().timestamp() + (ttl as i64);

    let claims = serde_json::json!({
        "sub": payload.external_user_id,
        "username": payload.username.unwrap_or_else(|| payload.external_user_id.clone()),
        "tenant_id": payload.tenant_id,
        "roles": payload.roles.unwrap_or_else(|| vec!["member".to_string()]),
        "email": payload.email,
        "exp": expires_at,
        "iat": Utc::now().timestamp(),
    });

    let payload_bytes = serde_json::to_vec(&claims).unwrap_or_default();
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload_bytes);
    let secret = get_secret_key();
    let signature_b64 = compute_signature(&secret, &payload_b64);

    let token = format!("lumis_embed_v1.{}.{}", payload_b64, signature_b64);

    (
        StatusCode::OK,
        Json(CreateEmbedTokenResponse {
            token,
            user_id: payload.external_user_id,
            expires_at,
        }),
    )
        .into_response()
}

pub async fn verify_embed_token(
    State(_state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token_str = headers
        .get("x-embed-token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|auth| auth.strip_prefix("Bearer ").map(|s| s.trim().to_string()))
        });

    let token = match token_str {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(VerifyEmbedTokenResponse {
                    valid: false,
                    user_id: None,
                    username: None,
                    tenant_id: None,
                    roles: Vec::new(),
                    expires_at: None,
                    error: Some("Missing embed token header (x-embed-token or Authorization: Bearer)".to_string()),
                }),
            );
        }
    };

    if !token.starts_with("lumis_embed_v1.") {
        return (
            StatusCode::UNAUTHORIZED,
            Json(VerifyEmbedTokenResponse {
                valid: false,
                user_id: None,
                username: None,
                tenant_id: None,
                roles: Vec::new(),
                expires_at: None,
                error: Some("Invalid token format prefix".to_string()),
            }),
        );
    }

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return (
            StatusCode::UNAUTHORIZED,
            Json(VerifyEmbedTokenResponse {
                valid: false,
                user_id: None,
                username: None,
                tenant_id: None,
                roles: Vec::new(),
                expires_at: None,
                error: Some("Malformed token segments".to_string()),
            }),
        );
    }

    let secret = get_secret_key();
    let expected_sig = compute_signature(&secret, parts[1]);
    if expected_sig != parts[2] {
        return (
            StatusCode::UNAUTHORIZED,
            Json(VerifyEmbedTokenResponse {
                valid: false,
                user_id: None,
                username: None,
                tenant_id: None,
                roles: Vec::new(),
                expires_at: None,
                error: Some("Invalid token signature".to_string()),
            }),
        );
    }

    let payload_bytes = match URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(VerifyEmbedTokenResponse {
                    valid: false,
                    user_id: None,
                    username: None,
                    tenant_id: None,
                    roles: Vec::new(),
                    expires_at: None,
                    error: Some("Failed to decode token base64".to_string()),
                }),
            );
        }
    };

    let claims: serde_json::Value = match serde_json::from_slice(&payload_bytes) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(VerifyEmbedTokenResponse {
                    valid: false,
                    user_id: None,
                    username: None,
                    tenant_id: None,
                    roles: Vec::new(),
                    expires_at: None,
                    error: Some("Invalid claims JSON structure".to_string()),
                }),
            );
        }
    };

    let exp = claims.get("exp").and_then(|e| e.as_i64()).unwrap_or(0);
    if exp < Utc::now().timestamp() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(VerifyEmbedTokenResponse {
                valid: false,
                user_id: None,
                username: None,
                tenant_id: None,
                roles: Vec::new(),
                expires_at: Some(exp),
                error: Some("Embed token has expired".to_string()),
            }),
        );
    }

    let roles = claims
        .get("roles")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    (
        StatusCode::OK,
        Json(VerifyEmbedTokenResponse {
            valid: true,
            user_id: claims.get("sub").and_then(|s| s.as_str()).map(|s| s.to_string()),
            username: claims.get("username").and_then(|s| s.as_str()).map(|s| s.to_string()),
            tenant_id: claims.get("tenant_id").and_then(|s| s.as_str()).map(|s| s.to_string()),
            roles,
            expires_at: Some(exp),
            error: None,
        }),
    )
}
