#![deny(unsafe_code)]

use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::mcp::{McpRiskTier, McpToolInvocationRecord, McpToolRecord};
use sqlx::{Row, SqlitePool};

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
    let payload_redacted_int = if inv.payload_redacted { 1 } else { 0 };

    sqlx::query(
        r#"
        INSERT INTO mcp_tool_invocations (
            id, tenant_id, user_id, server_id, tool_name, status,
            execution_time_ms, approval_token, payload_redacted, redaction_summary, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    .bind(payload_redacted_int)
    .bind(&inv.redaction_summary)
    .bind(created_at_str)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(())
}
