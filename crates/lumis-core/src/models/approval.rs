#![deny(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user::EnterpriseRole;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolApprovalPolicy {
    AutoApproved,
    RequiresApproval,
    RestrictedToRole(EnterpriseRole),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved {
        approved_by: String,
        timestamp: DateTime<Utc>,
    },
    Denied {
        denied_by: String,
        reason: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolApprovalRequest {
    pub token: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub requested_by: String,
    pub caller_role: EnterpriseRole,
    pub created_at: DateTime<Utc>,
    pub status: ApprovalStatus,
}

impl ToolApprovalRequest {
    pub fn is_pending(&self) -> bool {
        matches!(self.status, ApprovalStatus::Pending)
    }

    pub fn is_approved(&self) -> bool {
        matches!(self.status, ApprovalStatus::Approved { .. })
    }
}
