use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::mcp::{UserMcpOAuthTokenRecord, UserMcpPreferenceRecord};
use sqlx::{PgPool, Row};

pub async fn save_user_mcp_oauth_token(
    pool: &PgPool,
    token: &UserMcpOAuthTokenRecord,
) -> Result<UserMcpOAuthTokenRecord> {
    sqlx::query(
        r#"
        INSERT INTO user_mcp_oauth_tokens (
            id, user_id, server_id, access_token_encrypted, refresh_token_encrypted,
            token_type, scope, expires_at, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (user_id, server_id) DO UPDATE SET
            access_token_encrypted = EXCLUDED.access_token_encrypted,
            refresh_token_encrypted = EXCLUDED.refresh_token_encrypted,
            token_type = EXCLUDED.token_type,
            scope = EXCLUDED.scope,
            expires_at = EXCLUDED.expires_at,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(&token.id)
    .bind(&token.user_id)
    .bind(&token.server_id)
    .bind(&token.access_token_encrypted)
    .bind(&token.refresh_token_encrypted)
    .bind(&token.token_type)
    .bind(&token.scope)
    .bind(token.expires_at)
    .bind(token.created_at)
    .bind(token.updated_at)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(token.clone())
}

pub async fn get_user_mcp_oauth_token(
    pool: &PgPool,
    user_id: &str,
    server_id: &str,
) -> Result<Option<UserMcpOAuthTokenRecord>> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, server_id, access_token_encrypted, refresh_token_encrypted,
               token_type, scope, expires_at, created_at, updated_at
        FROM user_mcp_oauth_tokens
        WHERE user_id = $1 AND server_id = $2
        "#,
    )
    .bind(user_id)
    .bind(server_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let Some(r) = row else { return Ok(None); };

    let created_at: DateTime<Utc> = r.try_get("created_at").map_err(|e| LumisError::Database(e.to_string()))?;
    let updated_at: DateTime<Utc> = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(Some(UserMcpOAuthTokenRecord {
        id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
        user_id: r.try_get("user_id").map_err(|e| LumisError::Database(e.to_string()))?,
        server_id: r.try_get("server_id").map_err(|e| LumisError::Database(e.to_string()))?,
        access_token_encrypted: r.try_get("access_token_encrypted").map_err(|e| LumisError::Database(e.to_string()))?,
        refresh_token_encrypted: r.try_get("refresh_token_encrypted").unwrap_or(None),
        token_type: r.try_get("token_type").unwrap_or_else(|_| "Bearer".to_string()),
        scope: r.try_get("scope").unwrap_or(None),
        expires_at: r.try_get("expires_at").unwrap_or(None),
        created_at,
        updated_at,
    }))
}

pub async fn delete_user_mcp_oauth_token(
    pool: &PgPool,
    user_id: &str,
    server_id: &str,
) -> Result<bool> {
    let res = sqlx::query("DELETE FROM user_mcp_oauth_tokens WHERE user_id = $1 AND server_id = $2")
        .bind(user_id)
        .bind(server_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}

pub async fn get_user_mcp_preference(
    pool: &PgPool,
    user_id: &str,
    server_id: &str,
) -> Result<Option<bool>> {
    let row = sqlx::query(
        "SELECT is_enabled FROM user_mcp_preferences WHERE user_id = $1 AND server_id = $2",
    )
    .bind(user_id)
    .bind(server_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(row.map(|r| r.try_get("is_enabled").unwrap_or(true)))
}

pub async fn set_user_mcp_preference(
    pool: &PgPool,
    user_id: &str,
    server_id: &str,
    is_enabled: bool,
) -> Result<()> {
    let updated_at = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO user_mcp_preferences (user_id, server_id, is_enabled, updated_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, server_id) DO UPDATE SET
            is_enabled = EXCLUDED.is_enabled,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(user_id)
    .bind(server_id)
    .bind(is_enabled)
    .bind(updated_at)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(())
}

pub async fn list_user_mcp_preferences(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<UserMcpPreferenceRecord>> {
    let rows = sqlx::query(
        "SELECT user_id, server_id, is_enabled, updated_at FROM user_mcp_preferences WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        let created_at: DateTime<Utc> = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;
        result.push(UserMcpPreferenceRecord {
            user_id: r.try_get("user_id").map_err(|e| LumisError::Database(e.to_string()))?,
            server_id: r.try_get("server_id").map_err(|e| LumisError::Database(e.to_string()))?,
            is_enabled: r.try_get("is_enabled").unwrap_or(true),
            updated_at: created_at,
        });
    }

    Ok(result)
}
