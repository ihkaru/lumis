#![deny(unsafe_code)]

use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::mcp::{
    McpAuthType, McpRiskTier, McpServerRecord, McpTransportType,
};
use sqlx::{Row, SqlitePool};

pub use super::mcp_quota::{check_and_increment_mcp_quota, get_mcp_quota_usage};
pub use super::mcp_tools::{list_mcp_tools, log_mcp_invocation, sync_mcp_tools};

fn map_mcp_server_row(r: sqlx::sqlite::SqliteRow) -> Result<McpServerRecord> {
    let transport_str: String = r.try_get("transport_type").map_err(|e| LumisError::Database(e.to_string()))?;
    let auth_str: String = r.try_get("auth_type").map_err(|e| LumisError::Database(e.to_string()))?;
    let cmd_args_raw: Option<String> = r.try_get("command_args").unwrap_or(None);
    let command_args = cmd_args_raw.and_then(|s| serde_json::from_str(&s).ok());
    let is_enabled_int: i64 = r.try_get("is_enabled").unwrap_or(1);
    let risk_tier_int: i64 = r.try_get("risk_tier").unwrap_or(1);
    let scope_type: String = r.try_get("scope_type").unwrap_or_else(|_| "system".to_string());
    let owner_user_id: Option<String> = r.try_get("owner_user_id").unwrap_or(None);
    let is_mandatory_int: i64 = r.try_get("is_mandatory").unwrap_or(0);
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
    let max_memory_mb: i32 = r.try_get::<i64, _>("max_memory_mb").unwrap_or(256) as i32;
    let timeout_seconds: i32 = r.try_get::<i64, _>("timeout_seconds").unwrap_or(30) as i32;
    let dlp_enabled_int: i64 = r.try_get("dlp_enabled").unwrap_or(1);
    let dlp_rules_raw: Option<String> = r.try_get("dlp_rules").unwrap_or(None);
    let dlp_rules: Vec<String> = dlp_rules_raw
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| vec!["credit_card".to_string(), "nik".to_string(), "phone".to_string(), "email".to_string(), "jwt".to_string()]);
    let rate_limit_rpm: i32 = r.try_get::<i64, _>("rate_limit_rpm").unwrap_or(60) as i32;
    let daily_request_limit: i32 = r.try_get::<i64, _>("daily_request_limit").unwrap_or(1000) as i32;
    let kms_provider: String = r.try_get("kms_provider").unwrap_or_else(|_| "local".to_string());
    let kms_key_id: Option<String> = r.try_get("kms_key_id").unwrap_or(None);

    let created_at_str: String = r.try_get("created_at").map_err(|e| LumisError::Database(e.to_string()))?;
    let updated_at_str: String = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;

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
        is_enabled: is_enabled_int != 0,
        risk_tier: McpRiskTier::from_i16(risk_tier_int as i16),
        scope_type,
        owner_user_id,
        is_mandatory: is_mandatory_int != 0,
        allowed_roles,
        oauth_config,
        ssrf_policy,
        egress_allowlist,
        sandbox_type,
        max_memory_mb,
        timeout_seconds,
        dlp_enabled: dlp_enabled_int != 0,
        dlp_rules,
        rate_limit_rpm,
        daily_request_limit,
        kms_provider,
        kms_key_id,
        created_at: DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

pub async fn create_mcp_server(pool: &SqlitePool, s: &McpServerRecord) -> Result<McpServerRecord> {
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
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    .bind(if s.is_enabled { 1 } else { 0 })
    .bind(s.risk_tier.as_i16() as i64)
    .bind(&s.scope_type)
    .bind(&s.owner_user_id)
    .bind(if s.is_mandatory { 1 } else { 0 })
    .bind(allowed_roles_json)
    .bind(oauth_config_json)
    .bind(&s.ssrf_policy)
    .bind(egress_allowlist_json)
    .bind(&s.sandbox_type)
    .bind(s.max_memory_mb as i64)
    .bind(s.timeout_seconds as i64)
    .bind(if s.dlp_enabled { 1 } else { 0 })
    .bind(dlp_rules_json)
    .bind(s.rate_limit_rpm as i64)
    .bind(s.daily_request_limit as i64)
    .bind(&s.kms_provider)
    .bind(&s.kms_key_id)
    .bind(s.created_at.to_rfc3339())
    .bind(s.updated_at.to_rfc3339())
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(s.clone())
}

pub async fn get_mcp_server(pool: &SqlitePool, id: &str, tenant_id: &str) -> Result<Option<McpServerRecord>> {
    let row = sqlx::query(
        r#"
        SELECT id, tenant_id, created_by, name, display_title, description,
               transport_type, endpoint_url, command_bin, command_args, auth_type,
               encrypted_credentials, is_enabled, risk_tier, scope_type, owner_user_id,
               is_mandatory, allowed_roles, oauth_config, ssrf_policy, egress_allowlist,
               sandbox_type, max_memory_mb, timeout_seconds, dlp_enabled, dlp_rules,
               rate_limit_rpm, daily_request_limit, kms_provider, kms_key_id, created_at, updated_at
        FROM mcp_servers
        WHERE (id = ? OR name = ?) AND tenant_id = ?
        "#,
    )
    .bind(id)
    .bind(id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    row.map(map_mcp_server_row).transpose()
}

pub async fn list_mcp_servers(pool: &SqlitePool, tenant_id: &str) -> Result<Vec<McpServerRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT id, tenant_id, created_by, name, display_title, description,
               transport_type, endpoint_url, command_bin, command_args, auth_type,
               encrypted_credentials, is_enabled, risk_tier, scope_type, owner_user_id,
               is_mandatory, allowed_roles, oauth_config, ssrf_policy, egress_allowlist,
               sandbox_type, max_memory_mb, timeout_seconds, dlp_enabled, dlp_rules,
               rate_limit_rpm, daily_request_limit, kms_provider, kms_key_id, created_at, updated_at
        FROM mcp_servers
        WHERE tenant_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    rows.into_iter().map(map_mcp_server_row).collect()
}

pub async fn list_mcp_servers_for_user(
    pool: &SqlitePool,
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
        WHERE tenant_id = ?
          AND (
            (scope_type = 'system' AND (is_mandatory = 1 OR allowed_roles LIKE ?))
            OR
            (scope_type = 'user' AND owner_user_id = ?)
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

    rows.into_iter().map(map_mcp_server_row).collect()
}

pub async fn update_mcp_server(pool: &SqlitePool, s: &McpServerRecord) -> Result<McpServerRecord> {
    let cmd_args_json = s.command_args.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let allowed_roles_json = serde_json::to_string(&s.allowed_roles).unwrap_or_else(|_| "[]".to_string());
    let oauth_config_json = s.oauth_config.as_ref().map(|o| serde_json::to_string(o).unwrap_or_default());
    let egress_allowlist_json = s.egress_allowlist.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let dlp_rules_json = serde_json::to_string(&s.dlp_rules).unwrap_or_else(|_| "[]".to_string());

    sqlx::query(
        r#"
        UPDATE mcp_servers
        SET display_title = ?, description = ?, transport_type = ?, endpoint_url = ?,
            command_bin = ?, command_args = ?, auth_type = ?, encrypted_credentials = ?,
            is_enabled = ?, risk_tier = ?, is_mandatory = ?, allowed_roles = ?,
            oauth_config = ?, ssrf_policy = ?, egress_allowlist = ?, sandbox_type = ?,
            max_memory_mb = ?, timeout_seconds = ?, dlp_enabled = ?, dlp_rules = ?,
            rate_limit_rpm = ?, daily_request_limit = ?, kms_provider = ?, kms_key_id = ?,
            updated_at = ?
        WHERE id = ? AND tenant_id = ?
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
    .bind(if s.is_enabled { 1 } else { 0 })
    .bind(s.risk_tier.as_i16() as i64)
    .bind(if s.is_mandatory { 1 } else { 0 })
    .bind(allowed_roles_json)
    .bind(oauth_config_json)
    .bind(&s.ssrf_policy)
    .bind(egress_allowlist_json)
    .bind(&s.sandbox_type)
    .bind(s.max_memory_mb as i64)
    .bind(s.timeout_seconds as i64)
    .bind(if s.dlp_enabled { 1 } else { 0 })
    .bind(dlp_rules_json)
    .bind(s.rate_limit_rpm as i64)
    .bind(s.daily_request_limit as i64)
    .bind(&s.kms_provider)
    .bind(&s.kms_key_id)
    .bind(s.updated_at.to_rfc3339())
    .bind(&s.id)
    .bind(&s.tenant_id)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(s.clone())
}

pub async fn delete_mcp_server(pool: &SqlitePool, id: &str, tenant_id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM mcp_servers WHERE (id = ? OR name = ?) AND tenant_id = ?")
        .bind(id)
        .bind(id)
        .bind(tenant_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}
