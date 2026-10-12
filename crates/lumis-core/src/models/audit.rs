#![deny(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::user::EnterpriseRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventStatus {
    Success,
    Denied,
    Failure,
}

impl AuditEventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Denied => "denied",
            Self::Failure => "failure",
        }
    }
}

/// Centralized SIEM Security Audit Event.
/// Compatible with Elastic Common Schema (ECS), ArcSight CEF, and RFC 5424 Syslog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub tenant_id: String,
    pub actor_id: String,
    pub actor_role: EnterpriseRole,
    pub action: String,
    pub resource_id: Option<String>,
    pub status: AuditEventStatus,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: serde_json::Value,
}

impl AuditLogEvent {
    /// Formats the event into Elastic Common Schema (ECS v8.x) JSON.
    pub fn to_ecs_json(&self) -> serde_json::Value {
        serde_json::json!({
            "@timestamp": self.timestamp.to_rfc3339(),
            "ecs.version": "8.11.0",
            "event": {
                "id": self.event_id,
                "action": self.action,
                "outcome": self.status.as_str(),
                "category": ["authentication", "access", "configuration"],
                "type": ["info"],
            },
            "user": {
                "id": self.actor_id,
                "roles": [self.actor_role.as_str()],
            },
            "organization": {
                "id": self.tenant_id,
            },
            "client": {
                "ip": self.ip_address,
            },
            "user_agent": {
                "original": self.user_agent,
            },
            "labels": {
                "resource_id": self.resource_id,
            },
            "custom": self.details,
        })
    }

    /// Formats the event into ArcSight Common Event Format (CEF:0).
    pub fn to_cef_string(&self) -> String {
        let severity = match self.status {
            AuditEventStatus::Success => "3",
            AuditEventStatus::Denied => "7",
            AuditEventStatus::Failure => "8",
        };

        format!(
            "CEF:0|Lumis|LumisPlatform|1.0|{}|{}|{}|suser={} shost={} cs1={} cs1Label=TenantId outcome={}",
            self.action,
            self.action,
            severity,
            self.actor_id,
            self.ip_address.as_deref().unwrap_or("127.0.0.1"),
            self.tenant_id,
            self.status.as_str()
        )
    }

    /// Formats the event into Syslog RFC 5424 compliant string.
    pub fn to_syslog_rfc5424(&self) -> String {
        // Priority 134: Facility 16 (local0) * 8 + Severity 6 (informational)
        format!(
            "<134>1 {} lumis-server lumis-audit {} [lumis@32473 eventId=\"{}\" tenantId=\"{}\" actor=\"{}\"] {}",
            self.timestamp.to_rfc3339(),
            self.event_id,
            self.event_id,
            self.tenant_id,
            self.actor_id,
            self.action
        )
    }
}
