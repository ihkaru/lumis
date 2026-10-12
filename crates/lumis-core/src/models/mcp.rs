#![deny(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::user::EnterpriseRole;

/// Model Context Protocol (MCP 2026) Tool Definition.
/// Supports Enterprise Managed Authorization (EMA) with role gating.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub min_role: EnterpriseRole,
}

/// Request to invoke an MCP Tool synchronously.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallRequest {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// Result returned from executing an MCP Tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallResult {
    pub content: String,
    pub is_error: bool,
}

/// Task execution state for the MCP 2026 Asynchronous Tasks Extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpTaskState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Handle and status for an asynchronous long-running MCP task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTaskStatus {
    pub task_id: String,
    pub tool_name: String,
    pub state: McpTaskState,
    pub progress: Option<f32>,
    pub result: Option<McpToolCallResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
