#![deny(unsafe_code)]

use async_trait::async_trait;
use crate::error::Result;
use crate::models::{AuditLogEvent, EnterpriseRole};

/// Interface for Enterprise SIEM Audit Logger and SOC Killswitch.
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Ingests a structured audit log event.
    async fn log_event(&self, event: &AuditLogEvent) -> Result<()>;

    /// Queries recent audit log events for a given tenant, gated by caller role.
    async fn query_recent_events(
        &self,
        tenant_id: &str,
        caller_role: EnterpriseRole,
        limit: usize,
    ) -> Result<Vec<AuditLogEvent>>;

    /// Checks if the global emergency killswitch is active.
    fn is_killswitch_active(&self) -> bool;

    /// Sets or clears the emergency killswitch state.
    async fn set_killswitch(
        &self,
        active: bool,
        reason: &str,
        activated_by: &str,
    ) -> Result<()>;

    /// Returns current killswitch metadata (status, reason, activated_by).
    fn get_killswitch_status(&self) -> (bool, Option<String>, Option<String>);
}
