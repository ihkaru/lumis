use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnterpriseRole {
    Guest = 1,
    Member = 2,
    PowerUser = 3,
    TenantAdmin = 4,
    SuperAdmin = 5,
}

impl Default for EnterpriseRole {
    fn default() -> Self {
        Self::Member
    }
}

impl EnterpriseRole {
    pub fn level(&self) -> u8 {
        *self as u8
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SuperAdmin => "super_admin",
            Self::TenantAdmin => "tenant_admin",
            Self::PowerUser => "power_user",
            Self::Member => "member",
            Self::Guest => "guest",
        }
    }

    pub fn is_at_least(&self, required: EnterpriseRole) -> bool {
        self.level() >= required.level()
    }

    pub fn can_access_clearance(&self, tier: &ModelClearanceTier) -> bool {
        self.is_at_least(tier.required_role())
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "super_admin" | "superadmin" | "admin" | "ciso" | "root" | "system_admin" | "security_officer"
            | "administrator" => Self::SuperAdmin,
            "tenant_admin" | "tenantadmin" | "dept_admin" | "division_head" | "satker_admin" | "kepala_satker"
            | "manager" | "org_admin" => Self::TenantAdmin,
            "power_user" | "poweruser" | "operator" | "lead" | "developer" | "engineer" | "analyst"
            | "specialist" | "ppk" => Self::PowerUser,
            "member" | "pegawai" | "staff" | "employee" | "user" | "internal" | "asn" => {
                Self::Member
            }
            "guest" | "viewer" | "external" | "contractor" | "auditor" | "intern" | "tamu" => {
                Self::Guest
            }
            _ => Self::Guest,
        }
    }

    pub fn from_claims(roles: &[String]) -> Self {
        roles
            .iter()
            .map(|r| Self::from_str(r))
            .max()
            .unwrap_or(Self::Member)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelClearanceTier {
    TierBasic = 1,
    TierStandard = 2,
    TierAdvanced = 3,
    TierReasoning = 4,
}

impl ModelClearanceTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TierBasic => "tier_basic",
            Self::TierStandard => "tier_standard",
            Self::TierAdvanced => "tier_advanced",
            Self::TierReasoning => "tier_reasoning",
        }
    }

    pub fn required_role(&self) -> EnterpriseRole {
        match self {
            Self::TierBasic => EnterpriseRole::Guest,
            Self::TierStandard => EnterpriseRole::Member,
            Self::TierAdvanced => EnterpriseRole::PowerUser,
            Self::TierReasoning => EnterpriseRole::PowerUser,
        }
    }

    pub fn from_model_id(model_id: &str) -> Self {
        let m = model_id.to_lowercase();
        if m.contains("r1") || m.contains("reasoning") || m.contains("o1") || m.contains("o3") {
            Self::TierReasoning
        } else if m.contains("mini") || m.contains("flash") || m.contains("nano") || m.contains("haiku") {
            Self::TierBasic
        } else if m.contains("sonnet") || m.contains("coder") || m.contains("opus") {
            Self::TierAdvanced
        } else if m.contains("pro") || m.contains("4o") || m.contains("claude") {
            Self::TierStandard
        } else {
            Self::TierBasic
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: EnterpriseRole,
    pub tenant_id: Option<String>,
    pub preferences: serde_json::Value,
    pub is_active: bool,
    #[serde(default, skip_serializing)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn hash_password(password: &str) -> String {
        let salt = uuid::Uuid::new_v4().to_string();
        Self::hash_with_salt(password, &salt)
    }

    pub fn hash_with_salt(password: &str, salt: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(password.as_bytes());
        format!("{}:{:x}", salt, hasher.finalize())
    }

    pub fn verify_password(&self, password: &str) -> bool {
        match &self.password_hash {
            Some(stored) => {
                let parts: Vec<&str> = stored.split(':').collect();
                if parts.len() != 2 {
                    return false;
                }
                let calculated = Self::hash_with_salt(password, parts[0]);
                calculated == *stored
            }
            None => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub tenant_id: Option<String>,
    pub department: Option<String>,
    pub employee_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    pub id: String,
    pub user_id: String,
    pub provider: String, // "local", "saml", "oidc", "proxy", "scim"
    pub provider_subject_id: String, // immutable IdP ID (sub, NameID, NIP, or username)
    pub credential_data: Option<serde_json::Value>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

