use std::collections::HashMap;
use crate::error::{LumisError, Result};
use crate::models::{EnterpriseRole, ModelClearanceTier, UserClaims};

/// Interface-First IAM Claims Translator (EIP Message Translator).
/// Decouples ingress authentication proxies (Traefik, OAuth2-Proxy, Keycloak, Dex)
/// from core business authorization logic.
pub trait ClaimTranslator: Send + Sync {
    /// Extracts canonical UserClaims from incoming HTTP header map.
    fn extract_claims(&self, headers: &HashMap<String, String>) -> Result<UserClaims>;

    /// Resolves the effective monotonic EnterpriseRole.
    fn resolve_effective_role(&self, claims: &UserClaims) -> EnterpriseRole;

    /// Evaluates if a given role possesses sufficient clearance to invoke a model.
    fn evaluate_model_clearance(
        &self,
        role: EnterpriseRole,
        model_id: &str,
    ) -> Result<ModelClearanceTier>;
}

/// Default Enterprise Claim Translator supporting SAML 2.0, Active Directory/LDAP,
/// and standard OIDC headers.
#[derive(Debug, Clone, Default)]
pub struct DefaultClaimTranslator;

impl DefaultClaimTranslator {
    pub fn new() -> Self {
        Self
    }
}

impl ClaimTranslator for DefaultClaimTranslator {
    fn extract_claims(&self, headers: &HashMap<String, String>) -> Result<UserClaims> {
        // Resolve embed token claims if present (Authorization: Bearer lumis_embed_v1... or x-embed-token)
        let embed_token = headers
            .get("x-embed-token")
            .cloned()
            .or_else(|| {
                headers.get("authorization").and_then(|auth| {
                    auth.strip_prefix("Bearer ").map(|s| s.trim().to_string())
                })
            });

        let mut embed_claims = None;
        if let Some(ref token) = embed_token {
            if token.starts_with("lumis_embed_v1.") {
                let parts: Vec<&str> = token.split('.').collect();
                if parts.len() == 3 {
                    use base64::Engine;
                    if let Ok(decoded) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
                        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                            let exp = val.get("exp").and_then(|e| e.as_i64()).unwrap_or(0);
                            if exp > chrono::Utc::now().timestamp() {
                                embed_claims = Some(val);
                            }
                        }
                    }
                }
            }
        }

        // Resolve user identifier
        let user_id = embed_claims
            .as_ref()
            .and_then(|v| v.get("sub").and_then(|s| s.as_str()).map(|s| s.to_string()))
            .or_else(|| headers.get("x-embed-user").cloned())
            .or_else(|| headers.get("x-auth-request-user").cloned())
            .or_else(|| headers.get("x-forwarded-user").cloned())
            .or_else(|| headers.get("x-remote-user").cloned())
            .or_else(|| headers.get("x-authentik-username").cloned())
            .or_else(|| headers.get("x-forwarded-preferred-username").cloned())
            .or_else(|| headers.get("x-lumis-user").cloned())
            .or_else(|| {
                headers.get("cookie").and_then(|c_str| {
                    c_str.split(';').find_map(|pair| {
                        let mut kv = pair.trim().splitn(2, '=');
                        if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                            if (k == "lumis_user" || k == "session_user") && !v.is_empty() {
                                return Some(v.to_string());
                            }
                        }
                        None
                    })
                })
            })
            .unwrap_or_else(|| "anonymous".to_string());

        if user_id == "anonymous" {
            return Err(LumisError::Unauthorized(
                "Missing authenticated enterprise identity header".to_string(),
            ));
        }

        let username = embed_claims
            .as_ref()
            .and_then(|v| v.get("username").and_then(|s| s.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| user_id.clone());

        // Resolve email
        let email = embed_claims
            .as_ref()
            .and_then(|v| v.get("email").and_then(|s| s.as_str()).map(|s| s.to_string()))
            .or_else(|| headers.get("x-auth-request-email").cloned())
            .or_else(|| headers.get("x-forwarded-email").cloned())
            .or_else(|| headers.get("x-authentik-email").cloned());

        // Resolve roles/groups
        let embed_roles: Vec<String> = embed_claims
            .as_ref()
            .and_then(|v| v.get("roles").and_then(|r| r.as_array()))
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let roles: Vec<String> = if !embed_roles.is_empty() {
            embed_roles
        } else {
            let roles_raw = headers
                .get("x-auth-request-groups")
                .or_else(|| headers.get("x-forwarded-groups"))
                .or_else(|| headers.get("x-authentik-groups"))
                .or_else(|| headers.get("x-forwarded-roles"))
                .or_else(|| headers.get("x-user-roles"));

            match roles_raw {
                Some(raw) => raw
                    .split([',', ';'])
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                None => Vec::new(),
            }
        };

        // Resolve tenant
        let tenant_id = embed_claims
            .as_ref()
            .and_then(|v| v.get("tenant_id").and_then(|s| s.as_str()).map(|s| s.to_string()))
            .or_else(|| headers.get("x-auth-request-tenant").cloned())
            .or_else(|| headers.get("x-forwarded-tenant").cloned())
            .or_else(|| headers.get("x-tenant-id").cloned());

        // Resolve department
        let department = headers
            .get("x-department")
            .or_else(|| headers.get("x-forwarded-department"))
            .cloned();

        // Resolve employee ID / NIP
        let employee_id = headers
            .get("x-employee-id")
            .or_else(|| headers.get("x-nip"))
            .cloned();

        Ok(UserClaims {
            user_id,
            username,
            email,
            roles,
            tenant_id,
            department,
            employee_id,
        })
    }

    fn resolve_effective_role(&self, claims: &UserClaims) -> EnterpriseRole {
        if claims.roles.is_empty() {
            EnterpriseRole::Member
        } else {
            EnterpriseRole::from_claims(&claims.roles)
        }
    }

    fn evaluate_model_clearance(
        &self,
        role: EnterpriseRole,
        model_id: &str,
    ) -> Result<ModelClearanceTier> {
        let tier = ModelClearanceTier::from_model_id(model_id);
        if role.can_access_clearance(&tier) {
            Ok(tier)
        } else {
            Err(LumisError::Forbidden(format!(
                "Role '{}' (level {}) does not meet required clearance '{}' for model '{}'",
                role.as_str(),
                role.level(),
                tier.as_str(),
                model_id
            )))
        }
    }
}
