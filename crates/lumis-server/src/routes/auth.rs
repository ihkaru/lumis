#![deny(unsafe_code)]

use std::collections::HashMap;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::{EnterpriseRole, ModelClearanceTier, User, UserClaims};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub authenticated: bool,
    pub role: EnterpriseRole,
    pub role_level: u8,
    pub user: Option<User>,
    pub claims: Option<UserClaims>,
    pub max_clearance_tier: ModelClearanceTier,
}

#[derive(Debug, Serialize)]
pub struct RoleHierarchyInfo {
    pub role: EnterpriseRole,
    pub level: u8,
    pub canonical_name: &'static str,
    pub clearance_tier: ModelClearanceTier,
}

pub async fn auth_me(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let mut header_map = HashMap::new();
    for (k, v) in headers.iter() {
        if let Ok(str_val) = v.to_str() {
            header_map.insert(k.as_str().to_lowercase(), str_val.to_string());
        }
    }
    if let Some(val) = headers.get(&state.trusted_header).and_then(|v| v.to_str().ok()) {
        header_map.insert("x-forwarded-user".to_string(), val.to_string());
    }

    let claims = match state.claim_translator.extract_claims(&header_map) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "authenticated": false,
                    "error": "Missing authenticated enterprise identity header"
                })),
            )
                .into_response();
        }
    };

    // Database is the single source of truth for user and role
    let user_res = state.repo.get_user_by_username(&claims.username).await;
    let user = match user_res {
        Ok(Some(mut existing_user)) => {
            if !existing_user.is_active {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({
                        "authenticated": false,
                        "error": "Akun Anda telah dinonaktifkan oleh administrator atau sistem SCIM"
                    })),
                )
                    .into_response();
            }
            // SCIM JIT sync only if external identity provider supplied explicit groups
            if !claims.roles.is_empty() {
                let idp_role = state.claim_translator.resolve_effective_role(&claims);
                if existing_user.role != idp_role || existing_user.email != claims.email {
                    existing_user.role = idp_role;
                    existing_user.email = claims.email.clone();
                    existing_user.updated_at = chrono::Utc::now();
                    let _ = state.repo.upsert_user(&existing_user).await;
                }
            }
            existing_user
        }
        Ok(None) => {
            let idp_role = if !claims.roles.is_empty() {
                state.claim_translator.resolve_effective_role(&claims)
            } else {
                EnterpriseRole::Member
            };
            let now = chrono::Utc::now();
            let new_user = User {
                id: Uuid::new_v4().to_string(),
                username: claims.username.clone(),
                email: claims.email.clone(),
                role: idp_role,
                tenant_id: claims.tenant_id.clone(),
                preferences: serde_json::json!({}),
                is_active: true,
                password_hash: None,
                created_at: now,
                updated_at: now,
            };
            match state.repo.upsert_user(&new_user).await {
                Ok(created) => created,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                        .into_response();
                }
            }
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let effective_role = user.role;
    let max_clearance = match effective_role {
        EnterpriseRole::SuperAdmin | EnterpriseRole::TenantAdmin | EnterpriseRole::PowerUser => ModelClearanceTier::TierReasoning,
        EnterpriseRole::Member => ModelClearanceTier::TierStandard,
        EnterpriseRole::Guest => ModelClearanceTier::TierBasic,
    };

    let mut response = (
        StatusCode::OK,
        Json(AuthResponse {
            authenticated: true,
            role: user.role,
            role_level: user.role.level(),
            user: Some(user),
            claims: Some(claims),
            max_clearance_tier: max_clearance,
        }),
    )
        .into_response();

    if let Ok(val) = HeaderValue::from_str(effective_role.as_str()) { response.headers_mut().insert("x-forwarded-role", val); }
    if let Ok(val) = HeaderValue::from_str(max_clearance.as_str()) { response.headers_mut().insert("x-forwarded-tier", val); }
    if let Ok(val) = HeaderValue::from_str(&effective_role.level().to_string()) { response.headers_mut().insert("x-user-role-level", val); }

    response
}

pub async fn list_roles() -> impl IntoResponse {
    let hierarchy = vec![
        RoleHierarchyInfo { role: EnterpriseRole::SuperAdmin, level: 5, canonical_name: "super_admin", clearance_tier: ModelClearanceTier::TierReasoning },
        RoleHierarchyInfo { role: EnterpriseRole::TenantAdmin, level: 4, canonical_name: "tenant_admin", clearance_tier: ModelClearanceTier::TierReasoning },
        RoleHierarchyInfo { role: EnterpriseRole::PowerUser, level: 3, canonical_name: "power_user", clearance_tier: ModelClearanceTier::TierReasoning },
        RoleHierarchyInfo { role: EnterpriseRole::Member, level: 2, canonical_name: "member", clearance_tier: ModelClearanceTier::TierStandard },
        RoleHierarchyInfo { role: EnterpriseRole::Guest, level: 1, canonical_name: "guest", clearance_tier: ModelClearanceTier::TierBasic },
    ];
    (StatusCode::OK, Json(hierarchy))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let clean_user = payload.username.trim();
    if clean_user.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Username tidak boleh kosong" })),
        )
            .into_response();
    }

    let raw_pass = payload.password.as_deref().unwrap_or("").trim();

    let user_res = state.repo.get_user_by_username(clean_user).await;
    let user = match user_res {
        Ok(Some(mut u)) => {
            if !u.is_active {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({ "error": "Akun Anda telah dinonaktifkan oleh administrator atau sistem SCIM" })),
                )
                    .into_response();
            }
            if u.password_hash.is_some() {
                if !u.verify_password(raw_pass) {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({ "error": "Kata sandi salah" })),
                    )
                        .into_response();
                }
            } else if !raw_pass.is_empty() {
                u.password_hash = Some(User::hash_password(raw_pass));
                u.updated_at = chrono::Utc::now();
                let _ = state.repo.upsert_user(&u).await;
            }

            // Sync/link local identity record
            let now = chrono::Utc::now();
            let identity = lumis_core::models::UserIdentity {
                id: format!("idty-local-{}", u.id),
                user_id: u.id.clone(),
                provider: "local".to_string(),
                provider_subject_id: clean_user.to_string(),
                credential_data: u.password_hash.as_ref().map(|h| serde_json::json!({ "password_hash": h })),
                last_login_at: Some(now),
                created_at: u.created_at,
                updated_at: now,
            };
            let _ = state.repo.link_identity(&identity).await;

            u
        }
        Ok(None) => {
            let existing_users = state.repo.list_users(10).await.unwrap_or_default();
            let has_super_admin = existing_users.iter().any(|u| u.role == EnterpriseRole::SuperAdmin);
            let initial_role = if !has_super_admin && (clean_user == "admin" || clean_user == "superadmin") {
                EnterpriseRole::SuperAdmin
            } else {
                EnterpriseRole::Member
            };

            let now = chrono::Utc::now();
            let new_user = User {
                id: Uuid::new_v4().to_string(),
                username: clean_user.to_string(),
                email: None,
                role: initial_role,
                tenant_id: None,
                preferences: serde_json::json!({}),
                is_active: true,
                password_hash: if !raw_pass.is_empty() { Some(User::hash_password(raw_pass)) } else { None },
                created_at: now,
                updated_at: now,
            };
            let created = match state.repo.upsert_user(&new_user).await {
                Ok(created) => created,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
            };

            let identity = lumis_core::models::UserIdentity {
                id: format!("idty-local-{}", created.id),
                user_id: created.id.clone(),
                provider: "local".to_string(),
                provider_subject_id: clean_user.to_string(),
                credential_data: created.password_hash.as_ref().map(|h| serde_json::json!({ "password_hash": h })),
                last_login_at: Some(now),
                created_at: now,
                updated_at: now,
            };
            let _ = state.repo.link_identity(&identity).await;

            created
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let role = user.role;
    let max_clearance = match role {
        EnterpriseRole::SuperAdmin | EnterpriseRole::TenantAdmin | EnterpriseRole::PowerUser => ModelClearanceTier::TierReasoning,
        EnterpriseRole::Member => ModelClearanceTier::TierStandard,
        EnterpriseRole::Guest => ModelClearanceTier::TierBasic,
    };

    let auth_res = AuthResponse {
        authenticated: true,
        user: Some(user.clone()),
        role,
        role_level: role.level(),
        claims: Some(UserClaims {
            user_id: user.id.clone(),
            username: clean_user.to_string(),
            email: None,
            roles: vec![role.as_str().to_string()],
            tenant_id: None,
            department: None,
            employee_id: None,
        }),
        max_clearance_tier: max_clearance,
    };

    let cookie_val = format!("lumis_user={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000", clean_user);
    let mut response = (StatusCode::OK, Json(auth_res)).into_response();
    if let Ok(hdr) = HeaderValue::from_str(&cookie_val) {
        response.headers_mut().insert(axum::http::header::SET_COOKIE, hdr);
    }
    response
}

pub async fn logout() -> impl IntoResponse {
    let cookie_val = "lumis_user=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    let mut response = (StatusCode::OK, Json(serde_json::json!({ "status": "logged_out" }))).into_response();
    if let Ok(hdr) = HeaderValue::from_str(cookie_val) {
        response.headers_mut().insert(axum::http::header::SET_COOKIE, hdr);
    }
    response
}

pub use super::password::{change_password, ChangePasswordRequest};
