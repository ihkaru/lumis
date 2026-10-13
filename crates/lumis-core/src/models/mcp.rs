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

/// Supported transport protocols for external MCP servers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpTransportType {
    Sse,
    StreamableHttp,
    Stdio,
}

impl McpTransportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sse => "sse",
            Self::StreamableHttp => "streamable_http",
            Self::Stdio => "stdio",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "sse" => Some(Self::Sse),
            "streamable_http" => Some(Self::StreamableHttp),
            "stdio" => Some(Self::Stdio),
            _ => None,
        }
    }
}

/// Supported authentication schemes for MCP servers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpAuthType {
    None,
    Bearer,
    Basic,
    OAuth2,
    Obo,
}

impl McpAuthType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Bearer => "bearer",
            Self::Basic => "basic",
            Self::OAuth2 => "oauth2",
            Self::Obo => "obo",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Self::None),
            "bearer" => Some(Self::Bearer),
            "basic" => Some(Self::Basic),
            "oauth2" => Some(Self::OAuth2),
            "obo" => Some(Self::Obo),
            _ => None,
        }
    }
}

/// Security clearance and risk classification for MCP tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpRiskTier {
    Safe = 1,
    Mutation = 2,
    Destructive = 3,
}

impl McpRiskTier {
    pub fn as_i16(&self) -> i16 {
        *self as i16
    }

    pub fn from_i16(val: i16) -> Self {
        match val {
            2 => Self::Mutation,
            3 => Self::Destructive,
            _ => Self::Safe,
        }
    }
}

/// Configuration for OAuth 2.1 (RFC 6749bis / MCP 2026 Authorization).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpOAuthConfig {
    pub client_id: String,
    pub client_secret_encrypted: Option<String>,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub scopes: Vec<String>,
    pub resource_indicator: Option<String>,
    pub registration_mode: String, // "pre_registered", "cimd", "dcr"
}

/// Relational database record representing a registered MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerRecord {
    pub id: String,
    pub tenant_id: String,
    pub created_by: String,
    pub name: String,
    pub display_title: String,
    pub description: Option<String>,
    pub transport_type: McpTransportType,
    pub endpoint_url: Option<String>,
    pub command_bin: Option<String>,
    pub command_args: Option<Vec<String>>,
    pub auth_type: McpAuthType,
    pub encrypted_credentials: Option<String>,
    pub is_enabled: bool,
    pub risk_tier: McpRiskTier,
    pub scope_type: String, // "system" or "user"
    pub owner_user_id: Option<String>,
    pub is_mandatory: bool,
    pub allowed_roles: Vec<String>,
    pub oauth_config: Option<McpOAuthConfig>,
    pub ssrf_policy: String,
    pub egress_allowlist: Option<Vec<String>>,
    pub sandbox_type: String,
    pub max_memory_mb: i32,
    pub timeout_seconds: i32,
    pub dlp_enabled: bool,
    pub dlp_rules: Vec<String>,
    pub rate_limit_rpm: i32,
    pub daily_request_limit: i32,
    pub kms_provider: String,
    pub kms_key_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input payload to create a new MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerCreateInput {
    pub name: String,
    pub display_title: String,
    pub description: Option<String>,
    pub transport_type: McpTransportType,
    pub endpoint_url: Option<String>,
    pub command_bin: Option<String>,
    pub command_args: Option<Vec<String>>,
    pub auth_type: McpAuthType,
    pub raw_credentials: Option<String>,
    pub risk_tier: Option<McpRiskTier>,
    pub scope_type: Option<String>,
    pub owner_user_id: Option<String>,
    pub is_mandatory: Option<bool>,
    pub allowed_roles: Option<Vec<String>>,
    pub oauth_config: Option<McpOAuthConfig>,
    pub ssrf_policy: Option<String>,
    pub egress_allowlist: Option<Vec<String>>,
    pub sandbox_type: Option<String>,
    pub max_memory_mb: Option<i32>,
    pub timeout_seconds: Option<i32>,
    pub dlp_enabled: Option<bool>,
    pub dlp_rules: Option<Vec<String>>,
    pub rate_limit_rpm: Option<i32>,
    pub daily_request_limit: Option<i32>,
    pub kms_provider: Option<String>,
    pub kms_key_id: Option<String>,
}

/// Input payload to update an existing MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerUpdateInput {
    pub display_title: Option<String>,
    pub description: Option<String>,
    pub endpoint_url: Option<String>,
    pub raw_credentials: Option<String>,
    pub is_enabled: Option<bool>,
    pub risk_tier: Option<McpRiskTier>,
    pub is_mandatory: Option<bool>,
    pub allowed_roles: Option<Vec<String>>,
    pub oauth_config: Option<McpOAuthConfig>,
    pub ssrf_policy: Option<String>,
    pub egress_allowlist: Option<Vec<String>>,
    pub sandbox_type: Option<String>,
    pub max_memory_mb: Option<i32>,
    pub timeout_seconds: Option<i32>,
    pub dlp_enabled: Option<bool>,
    pub dlp_rules: Option<Vec<String>>,
    pub rate_limit_rpm: Option<i32>,
    pub daily_request_limit: Option<i32>,
    pub kms_provider: Option<String>,
    pub kms_key_id: Option<String>,
}

/// Relational token record for User-level MCP OAuth 2.1 access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMcpOAuthTokenRecord {
    pub id: String,
    pub user_id: String,
    pub server_id: String,
    pub access_token_encrypted: String,
    pub refresh_token_encrypted: Option<String>,
    pub token_type: String,
    pub scope: Option<String>,
    pub expires_at: Option<i64>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub idp_session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Relational preference record for per-user MCP server enable/disable toggle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMcpPreferenceRecord {
    pub user_id: String,
    pub server_id: String,
    pub is_enabled: bool,
    pub updated_at: DateTime<Utc>,
}

/// Relational database record representing an individual MCP tool discovered from a server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolRecord {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema_json: String,
    pub risk_tier: McpRiskTier,
    pub requires_approval: bool,
    pub updated_at: DateTime<Utc>,
}

/// Relational audit log record for MCP tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInvocationRecord {
    pub id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub server_id: String,
    pub tool_name: String,
    pub status: String,
    pub execution_time_ms: u32,
    pub approval_token: Option<String>,
    pub payload_redacted: bool,
    pub redaction_summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Relational quota tracking record for rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpUsageQuotaRecord {
    pub id: String,
    pub user_id: String,
    pub server_id: String,
    pub window_date: String,
    pub request_count: i32,
    pub updated_at: DateTime<Utc>,
}
