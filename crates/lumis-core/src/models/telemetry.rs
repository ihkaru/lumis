use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user::EnterpriseRole;

/// OpenTelemetry GenAI Semantic Convention (gen_ai.*) usage event record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageRecord {
    pub tenant_id: String,
    pub user_id: String,
    pub role: EnterpriseRole,
    pub model_id: String,
    pub provider: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// Result of evaluating FinOps token budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaCheckResult {
    /// Token usage allowed within allocated budget.
    Allowed { remaining: u64 },
    /// Token quota exceeded for role/tenant.
    Exceeded {
        limit: u64,
        used: u64,
        retry_after_secs: u64,
    },
}
