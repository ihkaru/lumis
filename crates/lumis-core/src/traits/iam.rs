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
        // Resolve user identifier
        let user_id = headers
            .get("x-auth-request-user")
            .or_else(|| headers.get("x-forwarded-user"))
            .or_else(|| headers.get("x-remote-user"))
            .or_else(|| headers.get("x-authentik-username"))
            .or_else(|| headers.get("x-forwarded-preferred-username"))
            .or_else(|| headers.get("x-lumis-user"))
            .cloned()
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

        let username = user_id.clone();

        // Resolve email
        let email = headers
            .get("x-auth-request-email")
            .or_else(|| headers.get("x-forwarded-email"))
            .or_else(|| headers.get("x-authentik-email"))
            .cloned();

        // Resolve roles/groups (supports comma and semicolon delimiters)
        let roles_raw = headers
            .get("x-auth-request-groups")
            .or_else(|| headers.get("x-forwarded-groups"))
            .or_else(|| headers.get("x-authentik-groups"))
            .or_else(|| headers.get("x-forwarded-roles"))
            .or_else(|| headers.get("x-user-roles"));

        let roles: Vec<String> = match roles_raw {
            Some(raw) => raw
                .split([',', ';'])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            None => Vec::new(),
        };

        // Resolve tenant
        let tenant_id = headers
            .get("x-auth-request-tenant")
            .or_else(|| headers.get("x-forwarded-tenant"))
            .or_else(|| headers.get("x-tenant-id"))
            .cloned();

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
