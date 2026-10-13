use std::collections::HashMap;
use axum::http::HeaderMap;
use chrono::Utc;
use lumis_core::models::{EnterpriseRole, User};
use uuid::Uuid;

use crate::state::AppState;

pub async fn resolve_claims_and_role(state: &AppState, headers: &HeaderMap) -> (String, EnterpriseRole) {
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
        if !claims.roles.is_empty() {
            let role = state.claim_translator.resolve_effective_role(&claims);
            (claims.username, role)
        } else {
            // Local / Cookie session: Database is the single source of truth
            match state.repo.get_user_by_username(&claims.username).await {
                Ok(Some(u)) => (claims.username, u.role),
                _ => (claims.username, EnterpriseRole::Member),
            }
        }
    } else {
        ("default_user".to_string(), EnterpriseRole::Member)
    }
}

pub async fn ensure_user(state: &AppState, username: &str, role: EnterpriseRole) -> Result<User, String> {
    ensure_federated_user(state, "proxy", username, username, role, None).await
}

pub async fn ensure_federated_user(
    state: &AppState,
    provider: &str,
    subject_id: &str,
    username: &str,
    role: EnterpriseRole,
    email: Option<String>,
) -> Result<User, String> {
    // 1. Try finding by federated identity (Immutable Subject Mapping)
    if let Ok(Some(idty)) = state.repo.find_identity(provider, subject_id).await {
        if let Ok(Some(u)) = state.repo.get_user_by_id(&idty.user_id).await {
            if !u.is_active {
                return Err("Akun pengguna dinonaktifkan".to_string());
            }
            return Ok(u);
        }
    }

    // 2. Try finding by username fallback
    let user = match state.repo.get_user_by_username(username).await {
        Ok(Some(u)) => {
            if !u.is_active {
                return Err("Akun pengguna dinonaktifkan".to_string());
            }
            u
        }
        Ok(None) => {
            let now = Utc::now();
            let new_user = User {
                id: Uuid::new_v4().to_string(),
                username: username.to_string(),
                email,
                role,
                tenant_id: None,
                preferences: serde_json::json!({}),
                is_active: true,
                password_hash: None,
                created_at: now,
                updated_at: now,
            };
            state.repo.upsert_user(&new_user).await.map_err(|e| e.to_string())?
        }
        Err(e) => return Err(e.to_string()),
    };

    // 3. Link federated identity
    let now = Utc::now();
    let identity = lumis_core::models::UserIdentity {
        id: format!("idty-{}-{}", provider, Uuid::new_v4()),
        user_id: user.id.clone(),
        provider: provider.to_string(),
        provider_subject_id: subject_id.to_string(),
        credential_data: Some(serde_json::json!({ "linked_at": now.to_rfc3339() })),
        last_login_at: Some(now),
        created_at: now,
        updated_at: now,
    };
    let _ = state.repo.link_identity(&identity).await;

    Ok(user)
}
