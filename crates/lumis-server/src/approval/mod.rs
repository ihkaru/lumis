#![deny(unsafe_code)]

use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::approval::{ApprovalStatus, ToolApprovalRequest};
use lumis_core::models::user::EnterpriseRole;
use lumis_core::traits::ToolApprovalManager;
use uuid::Uuid;

pub struct InMemoryApprovalManager {
    requests: RwLock<HashMap<String, ToolApprovalRequest>>,
}

impl InMemoryApprovalManager {
    pub fn new() -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolApprovalManager for InMemoryApprovalManager {
    async fn request_approval(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
        requested_by: &str,
        caller_role: EnterpriseRole,
    ) -> Result<ToolApprovalRequest> {
        let token = format!("appr-{}", Uuid::new_v4());
        let req = ToolApprovalRequest {
            token: token.clone(),
            tool_name: tool_name.to_string(),
            arguments: arguments.clone(),
            requested_by: requested_by.to_string(),
            caller_role,
            created_at: Utc::now(),
            status: ApprovalStatus::Pending,
        };

        let mut map = self.requests.write().unwrap();
        map.insert(token, req.clone());
        Ok(req)
    }

    async fn get_approval(&self, token: &str) -> Result<Option<ToolApprovalRequest>> {
        let map = self.requests.read().unwrap();
        Ok(map.get(token).cloned())
    }

    async fn list_pending_approvals(&self) -> Result<Vec<ToolApprovalRequest>> {
        let map = self.requests.read().unwrap();
        Ok(map.values().filter(|r| r.is_pending()).cloned().collect())
    }

    async fn approve(&self, token: &str, approved_by: &str) -> Result<ToolApprovalRequest> {
        let mut map = self.requests.write().unwrap();
        if let Some(req) = map.get_mut(token) {
            req.status = ApprovalStatus::Approved {
                approved_by: approved_by.to_string(),
                timestamp: Utc::now(),
            };
            Ok(req.clone())
        } else {
            Err(LumisError::NotFound(format!("Approval token '{}' not found", token)))
        }
    }

    async fn deny(&self, token: &str, denied_by: &str, reason: Option<&str>) -> Result<ToolApprovalRequest> {
        let mut map = self.requests.write().unwrap();
        if let Some(req) = map.get_mut(token) {
            req.status = ApprovalStatus::Denied {
                denied_by: denied_by.to_string(),
                reason: reason.map(|s| s.to_string()),
                timestamp: Utc::now(),
            };
            Ok(req.clone())
        } else {
            Err(LumisError::NotFound(format!("Approval token '{}' not found", token)))
        }
    }
}
