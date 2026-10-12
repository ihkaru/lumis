#![deny(unsafe_code)]

use async_trait::async_trait;

use crate::error::Result;
use crate::models::approval::ToolApprovalRequest;
use crate::models::user::EnterpriseRole;

#[async_trait]
pub trait ToolApprovalManager: Send + Sync {
    async fn request_approval(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
        requested_by: &str,
        caller_role: EnterpriseRole,
    ) -> Result<ToolApprovalRequest>;

    async fn get_approval(&self, token: &str) -> Result<Option<ToolApprovalRequest>>;
    async fn list_pending_approvals(&self) -> Result<Vec<ToolApprovalRequest>>;
    async fn approve(&self, token: &str, approved_by: &str) -> Result<ToolApprovalRequest>;
    async fn deny(&self, token: &str, denied_by: &str, reason: Option<&str>) -> Result<ToolApprovalRequest>;
}
