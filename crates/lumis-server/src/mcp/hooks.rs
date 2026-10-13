#![deny(unsafe_code)]

use std::sync::Arc;
use async_trait::async_trait;
use lumis_core::error::Result;
use lumis_core::models::User;
use lumis_core::traits::AuditLogger;
use regex::Regex;
use serde_json::Value;

/// Enterprise Lifecycle Hook Interface for MCP Tool Execution.
/// Allows enterprises to enforce DLP parameter scrubbing, SIEM audit emission,
/// and dynamic security policy verification before and after tools execute.
#[async_trait]
pub trait McpExecutionHook: Send + Sync {
    fn name(&self) -> &str;

    /// Invoked before tool execution. Can inspect, sanitize, or reject arguments.
    async fn before_call(
        &self,
        user: &User,
        tool: &str,
        args: &Value,
    ) -> Result<Value>;

    /// Invoked after tool execution completes successfully. Can sanitize output.
    async fn after_call(
        &self,
        user: &User,
        tool: &str,
        result: &Value,
    ) -> Result<Value>;

    /// Invoked when tool execution encounters an error.
    async fn on_error(&self, user: &User, tool: &str, error: &str);
}

/// Pipeline manager executing registered enterprise lifecycle hooks in sequence.
#[derive(Clone)]
pub struct EnterpriseHookPipeline {
    hooks: Vec<Arc<dyn McpExecutionHook>>,
}

impl EnterpriseHookPipeline {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn with_defaults(audit_logger: Arc<dyn AuditLogger>) -> Self {
        let mut pipeline = Self::new();
        pipeline.add_hook(Arc::new(DlpScrubbingHook::new()));
        pipeline.add_hook(Arc::new(AuditLogHook::new(audit_logger)));
        pipeline
    }

    pub fn add_hook(&mut self, hook: Arc<dyn McpExecutionHook>) {
        self.hooks.push(hook);
    }

    pub async fn execute_before(
        &self,
        user: &User,
        tool: &str,
        args: &Value,
    ) -> Result<Value> {
        let mut current_args = args.clone();
        for hook in &self.hooks {
            current_args = hook.before_call(user, tool, &current_args).await?;
        }
        Ok(current_args)
    }

    pub async fn execute_after(
        &self,
        user: &User,
        tool: &str,
        result: &Value,
    ) -> Result<Value> {
        let mut current_result = result.clone();
        for hook in &self.hooks {
            current_result = hook.after_call(user, tool, &current_result).await?;
        }
        Ok(current_result)
    }

    pub async fn execute_on_error(&self, user: &User, tool: &str, error: &str) {
        for hook in &self.hooks {
            hook.on_error(user, tool, error).await;
        }
    }
}

impl Default for EnterpriseHookPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// DLP Data Loss Prevention Hook. Redacts sensitive credentials, JWTs, card numbers,
/// and identity numbers in both tool arguments and execution outputs.
pub struct DlpScrubbingHook {
    patterns: Vec<(Regex, &'static str)>,
}

impl DlpScrubbingHook {
    pub fn new() -> Self {
        let patterns = vec![
            (Regex::new(r"ey[A-Za-z0-9-_]{10,}\.[A-Za-z0-9-_]{10,}\.[A-Za-z0-9-_]{10,}").unwrap(), "[REDACTED_JWT]"),
            (Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(), "[REDACTED_API_KEY]"),
            (Regex::new(r"\b[1-9]\d{15}\b").unwrap(), "[REDACTED_ID_NUMBER]"),
            (Regex::new(r"\b(?:\d{4}[ -]){3}\d{4}\b").unwrap(), "[REDACTED_CARD]"),
        ];
        Self { patterns }
    }

    fn scrub_value(&self, val: &Value) -> Value {
        match val {
            Value::String(s) => {
                let mut scrubbed = s.clone();
                for (regex, mask) in &self.patterns {
                    let rep: &str = *mask;
                    scrubbed = regex.replace_all(&scrubbed, rep).to_string();
                }
                Value::String(scrubbed)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|item| self.scrub_value(item)).collect()),
            Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    new_map.insert(k.clone(), self.scrub_value(v));
                }
                Value::Object(new_map)
            }
            other => other.clone(),
        }
    }
}

impl Default for DlpScrubbingHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl McpExecutionHook for DlpScrubbingHook {
    fn name(&self) -> &str {
        "dlp_scrubber"
    }

    async fn before_call(&self, _user: &User, _tool: &str, args: &Value) -> Result<Value> {
        Ok(self.scrub_value(args))
    }

    async fn after_call(&self, _user: &User, _tool: &str, result: &Value) -> Result<Value> {
        Ok(self.scrub_value(result))
    }

    async fn on_error(&self, _user: &User, _tool: &str, _error: &str) {}
}

/// Audit Log Hook. Emits structured security events to the enterprise audit subsystem.
pub struct AuditLogHook {
    audit_logger: Arc<dyn AuditLogger>,
}

impl AuditLogHook {
    pub fn new(audit_logger: Arc<dyn AuditLogger>) -> Self {
        Self { audit_logger }
    }
}

#[async_trait]
impl McpExecutionHook for AuditLogHook {
    fn name(&self) -> &str {
        "siem_audit_logger"
    }

    async fn before_call(&self, user: &User, tool: &str, _args: &Value) -> Result<Value> {
        tracing::info!(
            target: "lumis::audit::mcp",
            user_id = %user.id,
            username = %user.username,
            role = %user.role.as_str(),
            tool = %tool,
            "MCP Tool Invocation Initiated"
        );
        Ok(_args.clone())
    }

    async fn after_call(&self, user: &User, tool: &str, _result: &Value) -> Result<Value> {
        let event = lumis_core::models::AuditLogEvent {
            event_id: format!("audit_{}", uuid::Uuid::new_v4().simple()),
            timestamp: chrono::Utc::now(),
            tenant_id: user.tenant_id.clone().unwrap_or_else(|| "default".to_string()),
            actor_id: user.id.clone(),
            actor_role: user.role,
            action: format!("mcp_tool_execute:{}", tool),
            resource_id: Some(tool.to_string()),
            status: lumis_core::models::AuditEventStatus::Success,
            ip_address: None,
            user_agent: None,
            details: serde_json::json!({ "tool": tool, "status": "success" }),
        };
        let _ = self.audit_logger.log_event(&event).await;
        tracing::info!(
            target: "lumis::audit::mcp",
            user_id = %user.id,
            username = %user.username,
            tool = %tool,
            status = "success",
            "MCP Tool Invocation Succeeded"
        );
        Ok(_result.clone())
    }

    async fn on_error(&self, user: &User, tool: &str, error: &str) {
        let event = lumis_core::models::AuditLogEvent {
            event_id: format!("audit_{}", uuid::Uuid::new_v4().simple()),
            timestamp: chrono::Utc::now(),
            tenant_id: user.tenant_id.clone().unwrap_or_else(|| "default".to_string()),
            actor_id: user.id.clone(),
            actor_role: user.role,
            action: format!("mcp_tool_execute:{}", tool),
            resource_id: Some(tool.to_string()),
            status: lumis_core::models::AuditEventStatus::Failure,
            ip_address: None,
            user_agent: None,
            details: serde_json::json!({ "tool": tool, "error": error }),
        };
        let _ = self.audit_logger.log_event(&event).await;
        tracing::warn!(
            target: "lumis::audit::mcp",
            user_id = %user.id,
            username = %user.username,
            tool = %tool,
            error = %error,
            status = "failed",
            "MCP Tool Invocation Failed"
        );
    }
}
