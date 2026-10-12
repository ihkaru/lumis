use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lumis_core::error::Result;
use lumis_core::models::{EnterpriseRole, QuotaCheckResult, TokenUsageRecord};
use lumis_core::traits::TokenQuotaEngine;

struct WindowBucket {
    window_start: DateTime<Utc>,
    used_tokens: u64,
}

/// In-memory High Performance Sliding-Window Token Quota Engine.
/// Provides deterministic FinOps rate limiting per tenant and role.
pub struct InMemoryQuotaEngine {
    buckets: RwLock<HashMap<String, WindowBucket>>,
    window_duration_secs: i64,
}

impl InMemoryQuotaEngine {
    pub fn new() -> Self {
        Self {
            buckets: RwLock::new(HashMap::new()),
            window_duration_secs: 3600, // 1 hour sliding window
        }
    }

    #[allow(dead_code)]
    pub fn with_window_secs(window_secs: i64) -> Self {
        Self {
            buckets: RwLock::new(HashMap::new()),
            window_duration_secs: window_secs,
        }
    }

    pub fn role_quota_limit(role: EnterpriseRole) -> u64 {
        match role {
            EnterpriseRole::SuperAdmin => 10_000_000,
            EnterpriseRole::TenantAdmin => 1_000_000,
            EnterpriseRole::PowerUser => 200_000,
            EnterpriseRole::Member => 50_000,
            EnterpriseRole::Guest => 5_000,
        }
    }

    fn bucket_key(tenant_id: &str, role: EnterpriseRole) -> String {
        format!("{}:{}", tenant_id, role.as_str())
    }
}

impl Default for InMemoryQuotaEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenQuotaEngine for InMemoryQuotaEngine {
    async fn check_quota(
        &self,
        tenant_id: &str,
        _user_id: &str,
        role: EnterpriseRole,
        requested_tokens: u64,
    ) -> QuotaCheckResult {
        let key = Self::bucket_key(tenant_id, role);
        let limit = Self::role_quota_limit(role);
        let now = Utc::now();

        let mut buckets = self.buckets.write().unwrap();
        let bucket = buckets.entry(key).or_insert(WindowBucket {
            window_start: now,
            used_tokens: 0,
        });

        if now.signed_duration_since(bucket.window_start).num_seconds() >= self.window_duration_secs {
            bucket.window_start = now;
            bucket.used_tokens = 0;
        }

        if bucket.used_tokens + requested_tokens > limit {
            let elapsed = now.signed_duration_since(bucket.window_start).num_seconds();
            let retry_after = (self.window_duration_secs - elapsed).max(1) as u64;
            QuotaCheckResult::Exceeded {
                limit,
                used: bucket.used_tokens,
                retry_after_secs: retry_after,
            }
        } else {
            QuotaCheckResult::Allowed {
                remaining: limit - (bucket.used_tokens + requested_tokens),
            }
        }
    }

    async fn record_usage(&self, record: &TokenUsageRecord) -> Result<()> {
        let key = Self::bucket_key(&record.tenant_id, record.role);
        let now = Utc::now();
        let total_added = record.input_tokens + record.output_tokens;

        let mut buckets = self.buckets.write().unwrap();
        let bucket = buckets.entry(key).or_insert(WindowBucket {
            window_start: now,
            used_tokens: 0,
        });

        if now.signed_duration_since(bucket.window_start).num_seconds() >= self.window_duration_secs {
            bucket.window_start = now;
            bucket.used_tokens = 0;
        }

        bucket.used_tokens += total_added;
        Ok(())
    }

    async fn get_usage_summary(&self, tenant_id: &str) -> Result<serde_json::Value> {
        let buckets = self.buckets.read().unwrap();
        let mut summaries = serde_json::Map::new();

        for role in &[
            EnterpriseRole::SuperAdmin,
            EnterpriseRole::TenantAdmin,
            EnterpriseRole::PowerUser,
            EnterpriseRole::Member,
            EnterpriseRole::Guest,
        ] {
            let key = Self::bucket_key(tenant_id, *role);
            let limit = Self::role_quota_limit(*role);
            let used = buckets.get(&key).map(|b| b.used_tokens).unwrap_or(0);
            summaries.insert(
                role.as_str().to_string(),
                serde_json::json!({
                    "limit": limit,
                    "used": used,
                    "remaining": limit.saturating_sub(used),
                }),
            );
        }

        Ok(serde_json::Value::Object(summaries))
    }
}
