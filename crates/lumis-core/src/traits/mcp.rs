#![deny(unsafe_code)]

use std::sync::Arc;
use async_trait::async_trait;
use crate::error::Result;
use crate::models::{
    EnterpriseRole, McpTaskStatus, McpToolCallRequest, McpToolCallResult, McpToolDefinition,
};

/// Interface for executing an individual MCP Tool.
#[async_trait]
pub trait McpToolExecutor: Send + Sync {
    /// Canonical tool name (e.g. "sql_query", "weather_lookup").
    fn name(&self) -> &str;

    /// Schema specification and minimum clearance role required.
    fn definition(&self) -> McpToolDefinition;

    /// Executes the tool with provided arguments.
    async fn execute(
        &self,
        caller_role: EnterpriseRole,
        arguments: &serde_json::Value,
    ) -> Result<McpToolCallResult>;
}

/// Interface for the centralized MCP 2026 Registry and Task Manager.
#[async_trait]
pub trait McpRegistry: Send + Sync {
    /// Registers a tool executor into the runtime registry.
    async fn register_tool(&self, executor: Arc<dyn McpToolExecutor>) -> Result<()>;

    /// Unregisters a tool from the runtime registry.
    async fn unregister_tool(&self, _name: &str) -> Result<bool> {
        Ok(false)
    }

    /// Lists tools accessible to the caller based on their role.
    async fn list_tools(&self, caller_role: EnterpriseRole) -> Vec<McpToolDefinition>;

    /// Executes an MCP tool synchronously.
    async fn execute_tool(
        &self,
        caller_role: EnterpriseRole,
        request: &McpToolCallRequest,
    ) -> Result<McpToolCallResult>;

    /// Spawns an asynchronous task under the MCP 2026 Tasks Extension.
    async fn spawn_task(
        &self,
        caller_role: EnterpriseRole,
        request: &McpToolCallRequest,
    ) -> Result<McpTaskStatus>;

    /// Retrieves current status of an asynchronous MCP task.
    async fn get_task_status(&self, task_id: &str) -> Result<Option<McpTaskStatus>>;

    /// Cancels a running or queued asynchronous task.
    async fn cancel_task(&self, task_id: &str) -> Result<bool>;

    /// Returns the current catalog revision ETag for conditional caching (RFC 7232).
    fn catalog_etag(&self) -> String {
        "\"mcp-v1\"".to_string()
    }
}
