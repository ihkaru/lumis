use std::collections::HashMap;
use axum::http::HeaderMap;
use chrono::Utc;
use lumis_core::models::{EnterpriseRole, User};
use uuid::Uuid;

use crate::state::AppState;

pub fn resolve_claims_and_role(state: &AppState, headers: &HeaderMap) -> (String, EnterpriseRole) {
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

pub async fn ensure_user(state: &AppState, username: &str, role: EnterpriseRole) -> Result<User, String> {
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
