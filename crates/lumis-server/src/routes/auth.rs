use std::collections::HashMap;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::{EnterpriseRole, ModelClearanceTier, User, UserClaims};
use serde::Serialize;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub authenticated: bool,
    pub user: Option<User>,
    pub role: EnterpriseRole,
    pub role_level: u8,
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
    // 1. Convert HeaderMap to HashMap<String, String> for claim extraction
    let mut header_map = HashMap::new();
    for (k, v) in headers.iter() {
        if let Ok(str_val) = v.to_str() {
            header_map.insert(k.as_str().to_lowercase(), str_val.to_string());
        }
    }

    // Support custom trusted_header override
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

    let effective_role = state.claim_translator.resolve_effective_role(&claims);

    // 2. Resolve or JIT-provision user
    let user_res = state.repo.get_user_by_username(&claims.username).await;
    let user = match user_res {
        Ok(Some(mut existing_user)) => {
            // SCIM JIT sync: update role if changed
            if existing_user.role != effective_role || existing_user.email != claims.email {
                existing_user.role = effective_role;
                existing_user.email = claims.email.clone();
                existing_user.updated_at = chrono::Utc::now();
                let _ = state.repo.upsert_user(&existing_user).await;
            }
            existing_user
        }
        Ok(None) => {
            let now = chrono::Utc::now();
            let new_user = User {
                id: Uuid::new_v4().to_string(),
                username: claims.username.clone(),
                email: claims.email.clone(),
                role: effective_role,
                tenant_id: claims.tenant_id.clone(),
                preferences: serde_json::json!({}),
                is_active: true,
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

    let max_clearance = match effective_role {
        EnterpriseRole::SuperAdmin | EnterpriseRole::TenantAdmin | EnterpriseRole::PowerUser => {
            ModelClearanceTier::TierReasoning
        }
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

    // Inject standard Enterprise Context headers
    if let Ok(val) = HeaderValue::from_str(effective_role.as_str()) {
        response.headers_mut().insert("x-forwarded-role", val);
    }
    if let Ok(val) = HeaderValue::from_str(max_clearance.as_str()) {
        response.headers_mut().insert("x-forwarded-tier", val);
    }
    if let Ok(val) = HeaderValue::from_str(&effective_role.level().to_string()) {
        response.headers_mut().insert("x-user-role-level", val);
    }

    response
}

pub async fn list_roles() -> impl IntoResponse {
    let hierarchy = vec![
        RoleHierarchyInfo {
            role: EnterpriseRole::SuperAdmin,
            level: 5,
            canonical_name: "super_admin",
            clearance_tier: ModelClearanceTier::TierReasoning,
        },
        RoleHierarchyInfo {
            role: EnterpriseRole::TenantAdmin,
            level: 4,
            canonical_name: "tenant_admin",
            clearance_tier: ModelClearanceTier::TierReasoning,
        },
        RoleHierarchyInfo {
            role: EnterpriseRole::PowerUser,
            level: 3,
            canonical_name: "power_user",
            clearance_tier: ModelClearanceTier::TierReasoning,
        },
        RoleHierarchyInfo {
            role: EnterpriseRole::Member,
            level: 2,
            canonical_name: "member",
            clearance_tier: ModelClearanceTier::TierStandard,
        },
        RoleHierarchyInfo {
            role: EnterpriseRole::Guest,
            level: 1,
            canonical_name: "guest",
            clearance_tier: ModelClearanceTier::TierBasic,
        },
    ];

    (StatusCode::OK, Json(hierarchy))
}

#[derive(Debug, serde::Deserialize)]
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

    let role = if clean_user == "admin" || clean_user == "superadmin" {
        EnterpriseRole::SuperAdmin
    } else {
        EnterpriseRole::Member
    };

    let user = match crate::routes::chat::helpers::ensure_user(&state, clean_user, role).await {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };

    let max_clearance = match user.role {
        EnterpriseRole::SuperAdmin | EnterpriseRole::TenantAdmin | EnterpriseRole::PowerUser => {
            ModelClearanceTier::TierReasoning
        }
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

#[derive(Debug, serde::Deserialize)]
pub struct UpdateUserRoleRequest {
    pub username: String,
    pub role: String,
}

pub async fn list_admin_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_username, caller_role) = crate::routes::chat::helpers::resolve_claims_and_role(&state, &headers);
    if !caller_role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Only TenantAdmin or SuperAdmin can list enterprise users" })),
        )
            .into_response();
    }

    match state.repo.list_users(100).await {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn update_user_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateUserRoleRequest>,
) -> impl IntoResponse {
    let (_username, caller_role) = crate::routes::chat::helpers::resolve_claims_and_role(&state, &headers);
    if !caller_role.is_at_least(EnterpriseRole::TenantAdmin) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Only TenantAdmin or SuperAdmin can update user roles" })),
        )
            .into_response();
    }

    let target_role = EnterpriseRole::from_str(&payload.role);
    if target_role == EnterpriseRole::SuperAdmin && caller_role != EnterpriseRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Only SuperAdmin can promote users to SuperAdmin" })),
        )
            .into_response();
    }

    match state.repo.get_user_by_username(&payload.username).await {
        Ok(Some(mut user)) => {
            user.role = target_role;
            user.updated_at = chrono::Utc::now();
            match state.repo.upsert_user(&user).await {
                Ok(updated) => (StatusCode::OK, Json(updated)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Ok(None) => {
            let now = chrono::Utc::now();
            let new_u = lumis_core::models::User {
                id: uuid::Uuid::new_v4().to_string(),
                username: payload.username.clone(),
                email: None,
                role: target_role,
                tenant_id: None,
                preferences: serde_json::json!({}),
                is_active: true,
                created_at: now,
                updated_at: now,
            };
            match state.repo.upsert_user(&new_u).await {
                Ok(u) => (StatusCode::OK, Json(u)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
