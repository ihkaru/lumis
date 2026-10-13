use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::mcp::{
    McpAuthType, McpRiskTier, McpServerRecord, McpToolInvocationRecord, McpToolRecord,
    McpTransportType,
};
use sqlx::{PgPool, Row};

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
        created_at,
        updated_at,
    })
}

pub async fn create_mcp_server(pool: &PgPool, s: &McpServerRecord) -> Result<McpServerRecord> {
    let cmd_args_json = s.command_args.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let allowed_roles_json = serde_json::to_string(&s.allowed_roles).unwrap_or_else(|_| "[]".to_string());
    let oauth_config_json = s.oauth_config.as_ref().map(|o| serde_json::to_string(o).unwrap_or_default());
    let risk_tier_int = s.risk_tier.as_i16();

    sqlx::query(
        r#"
        INSERT INTO mcp_servers (
            id, tenant_id, created_by, name, display_title, description,
            transport_type, endpoint_url, command_bin, command_args, auth_type,
            encrypted_credentials, is_enabled, risk_tier, scope_type, owner_user_id,
            is_mandatory, allowed_roles, oauth_config, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
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
    .bind(risk_tier_int)
    .bind(&s.scope_type)
    .bind(&s.owner_user_id)
    .bind(s.is_mandatory)
    .bind(allowed_roles_json)
    .bind(oauth_config_json)
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
               is_mandatory, allowed_roles, oauth_config, created_at, updated_at
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
               is_mandatory, allowed_roles, oauth_config, created_at, updated_at
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
               is_mandatory, allowed_roles, oauth_config, created_at, updated_at
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
    let risk_tier_int = s.risk_tier.as_i16();

    sqlx::query(
        r#"
        UPDATE mcp_servers
        SET display_title = $1, description = $2, transport_type = $3, endpoint_url = $4,
            command_bin = $5, command_args = $6, auth_type = $7, encrypted_credentials = $8,
            is_enabled = $9, risk_tier = $10, is_mandatory = $11, allowed_roles = $12,
            oauth_config = $13, updated_at = $14
        WHERE id = $15 AND tenant_id = $16
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
    .bind(risk_tier_int)
    .bind(s.is_mandatory)
    .bind(allowed_roles_json)
    .bind(oauth_config_json)
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

pub async fn sync_mcp_tools(pool: &PgPool, server_id: &str, tools: &[McpToolRecord]) -> Result<()> {
    sqlx::query("DELETE FROM mcp_tools WHERE server_id = $1")
        .bind(server_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    for tool in tools {
        sqlx::query(
            r#"
            INSERT INTO mcp_tools (id, server_id, name, description, input_schema_json, risk_tier, requires_approval, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&tool.id)
        .bind(server_id)
        .bind(&tool.name)
        .bind(&tool.description)
        .bind(&tool.input_schema_json)
        .bind(tool.risk_tier.as_i16())
        .bind(tool.requires_approval)
        .bind(tool.updated_at)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;
    }

    Ok(())
}

pub async fn list_mcp_tools(pool: &PgPool, server_id: &str) -> Result<Vec<McpToolRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT id, server_id, name, description, input_schema_json, risk_tier, requires_approval, updated_at
        FROM mcp_tools
        WHERE server_id = $1
        ORDER BY name ASC
        "#,
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        let risk_int: i16 = r.try_get("risk_tier").unwrap_or(1);
        let requires_approval: bool = r.try_get("requires_approval").unwrap_or(false);
        let updated_at: DateTime<Utc> = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;

        result.push(McpToolRecord {
            id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
            server_id: r.try_get("server_id").map_err(|e| LumisError::Database(e.to_string()))?,
            name: r.try_get("name").map_err(|e| LumisError::Database(e.to_string()))?,
            description: r.try_get("description").unwrap_or(None),
            input_schema_json: r.try_get("input_schema_json").unwrap_or_else(|_| "{}".to_string()),
            risk_tier: McpRiskTier::from_i16(risk_int),
            requires_approval,
            updated_at,
        });
    }

    Ok(result)
}

pub async fn log_mcp_invocation(pool: &PgPool, inv: &McpToolInvocationRecord) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO mcp_tool_invocations (id, tenant_id, user_id, server_id, tool_name, status, execution_time_ms, approval_token, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(&inv.id)
    .bind(&inv.tenant_id)
    .bind(&inv.user_id)
    .bind(&inv.server_id)
    .bind(&inv.tool_name)
    .bind(&inv.status)
    .bind(inv.execution_time_ms as i32)
    .bind(&inv.approval_token)
    .bind(inv.created_at)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(())
}
