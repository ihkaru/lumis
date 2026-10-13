#![deny(unsafe_code)]

use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::mcp::{
    McpAuthType, McpRiskTier, McpServerRecord, McpTransportType,
};
use sqlx::{PgPool, Row};

pub use super::mcp_quota::{check_and_increment_mcp_quota, get_mcp_quota_usage};
pub use super::mcp_tools::{list_mcp_tools, log_mcp_invocation, sync_mcp_tools};

fn map_pg_mcp_server_row(r: sqlx::postgres::PgRow) -> Result<McpServerRecord> {
    let transport_str: String = r.try_get("transport_type").map_err(|e| LumisError::Database(e.to_string()))?;
    let auth_str: String = r.try_get("auth_type").map_err(|e| LumisError::Database(e.to_string()))?;
    let cmd_args_raw: Option<String> = r.try_get("command_args").unwrap_or(None);
    let command_args = cmd_args_raw.and_then(|s| serde_json::from_str(&s).ok());
    let is_enabled: bool = r.try_get("is_enabled").unwrap_or(true);
    let risk_tier_int: i16 = r.try_get("risk_tier").unwrap_or(1);
    let scope_type: String = r.try_get("scope_type").unwrap_or_else(|_| "system".to_string());
    let owner_user_id: Option<String> = r.try_get("owner_user_id").unwrap_or(None);
    let is_mandatory: bool = r.try_get("is_mandatory").unwrap_or(false);
    let allowed_roles_raw: Option<String> = r.try_get("allowed_roles").unwrap_or(None);
    let allowed_roles: Vec<String> = allowed_roles_raw
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| vec!["admin".to_string(), "user".to_string(), "auditor".to_string()]);
    let oauth_cfg_raw: Option<String> = r.try_get("oauth_config").unwrap_or(None);
    let oauth_config = oauth_cfg_raw.and_then(|s| serde_json::from_str(&s).ok());

    let ssrf_policy: String = r.try_get("ssrf_policy").unwrap_or_else(|_| "strict".to_string());
    let egress_allowlist_raw: Option<String> = r.try_get("egress_allowlist").unwrap_or(None);
    let egress_allowlist: Option<Vec<String>> = egress_allowlist_raw.and_then(|s| serde_json::from_str(&s).ok());
    let sandbox_type: String = r.try_get("sandbox_type").unwrap_or_else(|_| "restricted_process".to_string());
    let max_memory_mb: i32 = r.try_get("max_memory_mb").unwrap_or(256);
    let timeout_seconds: i32 = r.try_get("timeout_seconds").unwrap_or(30);
    let dlp_enabled: bool = r.try_get("dlp_enabled").unwrap_or(true);
    let dlp_rules_raw: Option<String> = r.try_get("dlp_rules").unwrap_or(None);
    let dlp_rules: Vec<String> = dlp_rules_raw
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| vec!["credit_card".to_string(), "nik".to_string(), "phone".to_string(), "email".to_string(), "jwt".to_string()]);
    let rate_limit_rpm: i32 = r.try_get("rate_limit_rpm").unwrap_or(60);
    let daily_request_limit: i32 = r.try_get("daily_request_limit").unwrap_or(1000);
    let kms_provider: String = r.try_get("kms_provider").unwrap_or_else(|_| "local".to_string());
    let kms_key_id: Option<String> = r.try_get("kms_key_id").unwrap_or(None);

    let created_at: DateTime<Utc> = r.try_get("created_at").map_err(|e| LumisError::Database(e.to_string()))?;
    let updated_at: DateTime<Utc> = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(McpServerRecord {
        id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
        tenant_id: r.try_get("tenant_id").map_err(|e| LumisError::Database(e.to_string()))?,
        created_by: r.try_get("created_by").map_err(|e| LumisError::Database(e.to_string()))?,
        name: r.try_get("name").map_err(|e| LumisError::Database(e.to_string()))?,
        display_title: r.try_get("display_title").map_err(|e| LumisError::Database(e.to_string()))?,
        description: r.try_get("description").unwrap_or(None),
        transport_type: McpTransportType::from_str_opt(&transport_str).unwrap_or(McpTransportType::Sse),
        endpoint_url: r.try_get("endpoint_url").unwrap_or(None),
        command_bin: r.try_get("command_bin").unwrap_or(None),
        command_args,
        auth_type: McpAuthType::from_str_opt(&auth_str).unwrap_or(McpAuthType::None),
        encrypted_credentials: r.try_get("encrypted_credentials").unwrap_or(None),
        is_enabled,
        risk_tier: McpRiskTier::from_i16(risk_tier_int),
        scope_type,
        owner_user_id,
        is_mandatory,
        allowed_roles,
        oauth_config,
        ssrf_policy,
        egress_allowlist,
        sandbox_type,
        max_memory_mb,
        timeout_seconds,
        dlp_enabled,
        dlp_rules,
        rate_limit_rpm,
        daily_request_limit,
        kms_provider,
        kms_key_id,
        created_at,
        updated_at,
    })
}

pub async fn create_mcp_server(pool: &PgPool, s: &McpServerRecord) -> Result<McpServerRecord> {
    let cmd_args_json = s.command_args.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let allowed_roles_json = serde_json::to_string(&s.allowed_roles).unwrap_or_else(|_| "[]".to_string());
    let oauth_config_json = s.oauth_config.as_ref().map(|o| serde_json::to_string(o).unwrap_or_default());
    let egress_allowlist_json = s.egress_allowlist.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let dlp_rules_json = serde_json::to_string(&s.dlp_rules).unwrap_or_else(|_| "[]".to_string());

    sqlx::query(
        r#"
        INSERT INTO mcp_servers (
            id, tenant_id, created_by, name, display_title, description,
            transport_type, endpoint_url, command_bin, command_args, auth_type,
            encrypted_credentials, is_enabled, risk_tier, scope_type, owner_user_id,
            is_mandatory, allowed_roles, oauth_config, ssrf_policy, egress_allowlist,
            sandbox_type, max_memory_mb, timeout_seconds, dlp_enabled, dlp_rules,
            rate_limit_rpm, daily_request_limit, kms_provider, kms_key_id, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
            $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32
        )
        "#,
    )
    .bind(&s.id)
    .bind(&s.tenant_id)
    .bind(&s.created_by)
    .bind(&s.name)
    .bind(&s.display_title)
    .bind(&s.description)
    .bind(s.transport_type.as_str())
    .bind(&s.endpoint_url)
    .bind(&s.command_bin)
    .bind(cmd_args_json)
    .bind(s.auth_type.as_str())
    .bind(&s.encrypted_credentials)
    .bind(s.is_enabled)
    .bind(s.risk_tier.as_i16())
    .bind(&s.scope_type)
    .bind(&s.owner_user_id)
    .bind(s.is_mandatory)
    .bind(allowed_roles_json)
    .bind(oauth_config_json)
    .bind(&s.ssrf_policy)
    .bind(egress_allowlist_json)
    .bind(&s.sandbox_type)
    .bind(s.max_memory_mb)
    .bind(s.timeout_seconds)
    .bind(s.dlp_enabled)
    .bind(dlp_rules_json)
    .bind(s.rate_limit_rpm)
    .bind(s.daily_request_limit)
    .bind(&s.kms_provider)
    .bind(&s.kms_key_id)
    .bind(s.created_at)
    .bind(s.updated_at)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(s.clone())
}

pub async fn get_mcp_server(pool: &PgPool, id: &str, tenant_id: &str) -> Result<Option<McpServerRecord>> {
    let row = sqlx::query(
        r#"
        SELECT id, tenant_id, created_by, name, display_title, description,
               transport_type, endpoint_url, command_bin, command_args, auth_type,
               encrypted_credentials, is_enabled, risk_tier, scope_type, owner_user_id,
               is_mandatory, allowed_roles, oauth_config, ssrf_policy, egress_allowlist,
               sandbox_type, max_memory_mb, timeout_seconds, dlp_enabled, dlp_rules,
               rate_limit_rpm, daily_request_limit, kms_provider, kms_key_id, created_at, updated_at
        FROM mcp_servers
        WHERE (id = $1 OR name = $1) AND tenant_id = $2
        "#,
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    row.map(map_pg_mcp_server_row).transpose()
}

pub async fn list_mcp_servers(pool: &PgPool, tenant_id: &str) -> Result<Vec<McpServerRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT id, tenant_id, created_by, name, display_title, description,
               transport_type, endpoint_url, command_bin, command_args, auth_type,
               encrypted_credentials, is_enabled, risk_tier, scope_type, owner_user_id,
               is_mandatory, allowed_roles, oauth_config, ssrf_policy, egress_allowlist,
               sandbox_type, max_memory_mb, timeout_seconds, dlp_enabled, dlp_rules,
               rate_limit_rpm, daily_request_limit, kms_provider, kms_key_id, created_at, updated_at
        FROM mcp_servers
        WHERE tenant_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    rows.into_iter().map(map_pg_mcp_server_row).collect()
}

pub async fn list_mcp_servers_for_user(
    pool: &PgPool,
    tenant_id: &str,
    user_id: &str,
    user_role: &str,
) -> Result<Vec<McpServerRecord>> {
    let role_pattern = format!("%\"{}\"%", user_role);
    let rows = sqlx::query(
        r#"
        SELECT id, tenant_id, created_by, name, display_title, description,
               transport_type, endpoint_url, command_bin, command_args, auth_type,
               encrypted_credentials, is_enabled, risk_tier, scope_type, owner_user_id,
               is_mandatory, allowed_roles, oauth_config, ssrf_policy, egress_allowlist,
               sandbox_type, max_memory_mb, timeout_seconds, dlp_enabled, dlp_rules,
               rate_limit_rpm, daily_request_limit, kms_provider, kms_key_id, created_at, updated_at
        FROM mcp_servers
        WHERE tenant_id = $1
          AND (
            (scope_type = 'system' AND (is_mandatory = TRUE OR allowed_roles LIKE $2))
            OR
            (scope_type = 'user' AND owner_user_id = $3)
          )
        ORDER BY is_mandatory DESC, created_at ASC
        "#,
    )
    .bind(tenant_id)
    .bind(role_pattern)
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    rows.into_iter().map(map_pg_mcp_server_row).collect()
}

pub async fn update_mcp_server(pool: &PgPool, s: &McpServerRecord) -> Result<McpServerRecord> {
    let cmd_args_json = s.command_args.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let allowed_roles_json = serde_json::to_string(&s.allowed_roles).unwrap_or_else(|_| "[]".to_string());
    let oauth_config_json = s.oauth_config.as_ref().map(|o| serde_json::to_string(o).unwrap_or_default());
    let egress_allowlist_json = s.egress_allowlist.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let dlp_rules_json = serde_json::to_string(&s.dlp_rules).unwrap_or_else(|_| "[]".to_string());

    sqlx::query(
        r#"
        UPDATE mcp_servers
        SET display_title = $1, description = $2, transport_type = $3, endpoint_url = $4,
            command_bin = $5, command_args = $6, auth_type = $7, encrypted_credentials = $8,
            is_enabled = $9, risk_tier = $10, is_mandatory = $11, allowed_roles = $12,
            oauth_config = $13, ssrf_policy = $14, egress_allowlist = $15, sandbox_type = $16,
            max_memory_mb = $17, timeout_seconds = $18, dlp_enabled = $19, dlp_rules = $20,
            rate_limit_rpm = $21, daily_request_limit = $22, kms_provider = $23, kms_key_id = $24,
            updated_at = $25
        WHERE id = $26 AND tenant_id = $27
        "#,
    )
    .bind(&s.display_title)
    .bind(&s.description)
    .bind(s.transport_type.as_str())
    .bind(&s.endpoint_url)
    .bind(&s.command_bin)
    .bind(cmd_args_json)
    .bind(s.auth_type.as_str())
    .bind(&s.encrypted_credentials)
    .bind(s.is_enabled)
    .bind(s.risk_tier.as_i16())
    .bind(s.is_mandatory)
    .bind(allowed_roles_json)
    .bind(oauth_config_json)
    .bind(&s.ssrf_policy)
    .bind(egress_allowlist_json)
    .bind(&s.sandbox_type)
    .bind(s.max_memory_mb)
    .bind(s.timeout_seconds)
    .bind(s.dlp_enabled)
    .bind(dlp_rules_json)
    .bind(s.rate_limit_rpm)
    .bind(s.daily_request_limit)
    .bind(&s.kms_provider)
    .bind(&s.kms_key_id)
    .bind(s.updated_at)
    .bind(&s.id)
    .bind(&s.tenant_id)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(s.clone())
}

pub async fn delete_mcp_server(pool: &PgPool, id: &str, tenant_id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM mcp_servers WHERE (id = $1 OR name = $1) AND tenant_id = $2")
        .bind(id)
        .bind(tenant_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}
