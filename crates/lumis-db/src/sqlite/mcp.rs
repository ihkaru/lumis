use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::mcp::{
    McpAuthType, McpRiskTier, McpServerRecord, McpToolInvocationRecord, McpToolRecord,
    McpTransportType,
};
use sqlx::{Row, SqlitePool};

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
    let is_enabled_int = if s.is_enabled { 1 } else { 0 };
    let is_mandatory_int = if s.is_mandatory { 1 } else { 0 };
    let risk_tier_int = s.risk_tier.as_i16() as i64;
    let created_at_str = s.created_at.to_rfc3339();
    let updated_at_str = s.updated_at.to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO mcp_servers (
            id, tenant_id, created_by, name, display_title, description,
            transport_type, endpoint_url, command_bin, command_args, auth_type,
            encrypted_credentials, is_enabled, risk_tier, scope_type, owner_user_id,
            is_mandatory, allowed_roles, oauth_config, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    .bind(is_enabled_int)
    .bind(risk_tier_int)
    .bind(&s.scope_type)
    .bind(&s.owner_user_id)
    .bind(is_mandatory_int)
    .bind(allowed_roles_json)
    .bind(oauth_config_json)
    .bind(created_at_str)
    .bind(updated_at_str)
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
               is_mandatory, allowed_roles, oauth_config, created_at, updated_at
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
               is_mandatory, allowed_roles, oauth_config, created_at, updated_at
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
               is_mandatory, allowed_roles, oauth_config, created_at, updated_at
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
    let is_enabled_int = if s.is_enabled { 1 } else { 0 };
    let is_mandatory_int = if s.is_mandatory { 1 } else { 0 };
    let risk_tier_int = s.risk_tier.as_i16() as i64;
    let updated_at_str = s.updated_at.to_rfc3339();

    sqlx::query(
        r#"
        UPDATE mcp_servers
        SET display_title = ?, description = ?, transport_type = ?, endpoint_url = ?,
            command_bin = ?, command_args = ?, auth_type = ?, encrypted_credentials = ?,
            is_enabled = ?, risk_tier = ?, is_mandatory = ?, allowed_roles = ?,
            oauth_config = ?, updated_at = ?
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
    .bind(is_enabled_int)
    .bind(risk_tier_int)
    .bind(is_mandatory_int)
    .bind(allowed_roles_json)
    .bind(oauth_config_json)
    .bind(updated_at_str)
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

pub async fn sync_mcp_tools(pool: &SqlitePool, server_id: &str, tools: &[McpToolRecord]) -> Result<()> {
    sqlx::query("DELETE FROM mcp_tools WHERE server_id = ?")
        .bind(server_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    for tool in tools {
        let req_appr_int = if tool.requires_approval { 1 } else { 0 };
        let updated_at_str = tool.updated_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO mcp_tools (id, server_id, name, description, input_schema_json, risk_tier, requires_approval, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&tool.id)
        .bind(server_id)
        .bind(&tool.name)
        .bind(&tool.description)
        .bind(&tool.input_schema_json)
        .bind(tool.risk_tier.as_i16() as i64)
        .bind(req_appr_int)
        .bind(updated_at_str)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;
    }

    Ok(())
}

pub async fn list_mcp_tools(pool: &SqlitePool, server_id: &str) -> Result<Vec<McpToolRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT id, server_id, name, description, input_schema_json, risk_tier, requires_approval, updated_at
        FROM mcp_tools
        WHERE server_id = ?
        ORDER BY name ASC
        "#,
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        let risk_int: i64 = r.try_get("risk_tier").unwrap_or(1);
        let req_appr_int: i64 = r.try_get("requires_approval").unwrap_or(0);
        let updated_at_str: String = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;

        result.push(McpToolRecord {
            id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
            server_id: r.try_get("server_id").map_err(|e| LumisError::Database(e.to_string()))?,
            name: r.try_get("name").map_err(|e| LumisError::Database(e.to_string()))?,
            description: r.try_get("description").unwrap_or(None),
            input_schema_json: r.try_get("input_schema_json").unwrap_or_else(|_| "{}".to_string()),
            risk_tier: McpRiskTier::from_i16(risk_int as i16),
            requires_approval: req_appr_int != 0,
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        });
    }

    Ok(result)
}

pub async fn log_mcp_invocation(pool: &SqlitePool, inv: &McpToolInvocationRecord) -> Result<()> {
    let created_at_str = inv.created_at.to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO mcp_tool_invocations (id, tenant_id, user_id, server_id, tool_name, status, execution_time_ms, approval_token, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&inv.id)
    .bind(&inv.tenant_id)
    .bind(&inv.user_id)
    .bind(&inv.server_id)
    .bind(&inv.tool_name)
    .bind(&inv.status)
    .bind(inv.execution_time_ms as i64)
    .bind(&inv.approval_token)
    .bind(created_at_str)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(())
}
