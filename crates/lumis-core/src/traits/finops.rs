use async_trait::async_trait;
use crate::error::Result;
use crate::models::{EnterpriseRole, QuotaCheckResult, TokenUsageRecord};

/// Token Quota & Budget Enforcement Contract (FinOps).
/// Enforces sliding-window token consumption per tenant/role.
#[async_trait]
pub trait TokenQuotaEngine: Send + Sync {
    /// Evaluates if an operation with requested tokens is permitted under the current quota window.
    async fn check_quota(
        &self,
        tenant_id: &str,
        user_id: &str,
        role: EnterpriseRole,
        requested_tokens: u64,
    ) -> QuotaCheckResult;

    /// Records consumed tokens for the given tenant and role.
    async fn record_usage(&self, record: &TokenUsageRecord) -> Result<()>;

    /// Returns high-level usage and limit summary for a tenant.
    async fn get_usage_summary(&self, tenant_id: &str) -> Result<serde_json::Value>;
}

/// OpenTelemetry GenAI Semantic Conventions Telemetry Collector.
/// Renders standard gen_ai.* metric events into Prometheus exposition format.
pub trait TelemetryCollector: Send + Sync {
    /// Ingests a new GenAI invocation telemetry event.
    fn record_gen_ai_event(&self, record: &TokenUsageRecord);

    /// Formats all collected metrics into standard Prometheus text format.
    fn render_prometheus_metrics(&self) -> String;
}
