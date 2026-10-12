#![deny(unsafe_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use async_trait::async_trait;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{AuditLogEvent, EnterpriseRole};
use lumis_core::traits::AuditLogger;

/// High-Performance In-Memory SIEM Audit Logger and Emergency SOC Killswitch Engine.
/// Stores ring-buffered structured audit events and provides sub-millisecond killswitch.
pub struct InMemoryAuditLogger {
    events: RwLock<Vec<AuditLogEvent>>,
    killswitch: AtomicBool,
    killswitch_reason: RwLock<Option<String>>,
    killswitch_by: RwLock<Option<String>>,
    capacity: usize,
}

impl InMemoryAuditLogger {
    pub fn new() -> Self {
        Self::with_capacity(10_000)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: RwLock::new(Vec::with_capacity(1000)),
            killswitch: AtomicBool::new(false),
            killswitch_reason: RwLock::new(None),
            killswitch_by: RwLock::new(None),
            capacity,
        }
    }
}

impl Default for InMemoryAuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditLogger for InMemoryAuditLogger {
    async fn log_event(&self, event: &AuditLogEvent) -> Result<()> {
        let mut events = self.events.write().unwrap();
        if events.len() >= self.capacity {
            // Evict oldest event (FIFO ring buffer)
            events.remove(0);
        }
        events.push(event.clone());
        Ok(())
    }

    async fn query_recent_events(
        &self,
        tenant_id: &str,
        caller_role: EnterpriseRole,
        limit: usize,
    ) -> Result<Vec<AuditLogEvent>> {
        // PEP Check: Only TenantAdmin and SuperAdmin can inspect SIEM audit logs
        if !caller_role.is_at_least(EnterpriseRole::TenantAdmin) {
            return Err(LumisError::Forbidden(
                "Access to SIEM audit logs requires TenantAdmin or SuperAdmin clearance".to_string(),
            ));
        }

        let events = self.events.read().unwrap();
        let matched: Vec<AuditLogEvent> = events
            .iter()
            .rev()
            .filter(|e| e.tenant_id == tenant_id || caller_role == EnterpriseRole::SuperAdmin)
            .take(limit)
            .cloned()
            .collect();

        Ok(matched)
    }

    fn is_killswitch_active(&self) -> bool {
        self.killswitch.load(Ordering::SeqCst)
    }

    async fn set_killswitch(
        &self,
        active: bool,
        reason: &str,
        activated_by: &str,
    ) -> Result<()> {
        self.killswitch.store(active, Ordering::SeqCst);
        {
            let mut r = self.killswitch_reason.write().unwrap();
            *r = if active { Some(reason.to_string()) } else { None };
        }
        {
            let mut b = self.killswitch_by.write().unwrap();
            *b = if active { Some(activated_by.to_string()) } else { None };
        }
        Ok(())
    }

    fn get_killswitch_status(&self) -> (bool, Option<String>, Option<String>) {
        let active = self.is_killswitch_active();
        let reason = self.killswitch_reason.read().unwrap().clone();
        let by = self.killswitch_by.read().unwrap().clone();
        (active, reason, by)
    }
}
