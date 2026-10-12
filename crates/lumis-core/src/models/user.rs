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

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "super_admin" | "admin" | "ciso" | "root" | "system_admin" => Self::SuperAdmin,
            "tenant_admin" | "dept_admin" | "satker_admin" | "manager" => Self::TenantAdmin,
            "power_user" | "developer" | "engineer" | "analyst" | "operator" => Self::PowerUser,
            "member" | "pegawai" | "staff" | "employee" | "user" => Self::Member,
            _ => Self::Guest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelClearanceTier {
    TierBasic = 1,
    TierStandard = 2,
    TierAdvanced = 3,
    TierReasoning = 4,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub tenant_id: Option<String>,
}
