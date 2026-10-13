use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::mcp::{UserMcpOAuthTokenRecord, UserMcpPreferenceRecord};
use sqlx::{Row, SqlitePool};

pub async fn save_user_mcp_oauth_token(
    pool: &SqlitePool,
    token: &UserMcpOAuthTokenRecord,
) -> Result<UserMcpOAuthTokenRecord> {
    let created_at_str = token.created_at.to_rfc3339();
    let updated_at_str = token.updated_at.to_rfc3339();
    let is_revoked_int = if token.is_revoked { 1 } else { 0 };
    let revoked_at_str = token.revoked_at.map(|dt| dt.to_rfc3339());

    sqlx::query(
        r#"
        INSERT INTO user_mcp_oauth_tokens (
            id, user_id, server_id, access_token_encrypted, refresh_token_encrypted,
            token_type, scope, expires_at, is_revoked, revoked_at, idp_session_id,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT (user_id, server_id) DO UPDATE SET
            access_token_encrypted = excluded.access_token_encrypted,
            refresh_token_encrypted = excluded.refresh_token_encrypted,
            token_type = excluded.token_type,
            scope = excluded.scope,
            expires_at = excluded.expires_at,
            is_revoked = excluded.is_revoked,
            revoked_at = excluded.revoked_at,
            idp_session_id = excluded.idp_session_id,
            updated_at = excluded.updated_at
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
    .bind(is_revoked_int)
    .bind(revoked_at_str)
    .bind(&token.idp_session_id)
    .bind(created_at_str)
    .bind(updated_at_str)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(token.clone())
}

pub async fn get_user_mcp_oauth_token(
    pool: &SqlitePool,
    user_id: &str,
    server_id: &str,
) -> Result<Option<UserMcpOAuthTokenRecord>> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, server_id, access_token_encrypted, refresh_token_encrypted,
               token_type, scope, expires_at, is_revoked, revoked_at, idp_session_id,
               created_at, updated_at
        FROM user_mcp_oauth_tokens
        WHERE user_id = ? AND server_id = ?
        "#,
    )
    .bind(user_id)
    .bind(server_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let Some(r) = row else { return Ok(None); };

    let is_revoked_int: i64 = r.try_get("is_revoked").unwrap_or(0);
    let revoked_at_raw: Option<String> = r.try_get("revoked_at").unwrap_or(None);
    let revoked_at = revoked_at_raw.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));
    let created_at_str: String = r.try_get("created_at").map_err(|e| LumisError::Database(e.to_string()))?;
    let updated_at_str: String = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(Some(UserMcpOAuthTokenRecord {
        id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
        user_id: r.try_get("user_id").map_err(|e| LumisError::Database(e.to_string()))?,
        server_id: r.try_get("server_id").map_err(|e| LumisError::Database(e.to_string()))?,
        access_token_encrypted: r.try_get("access_token_encrypted").map_err(|e| LumisError::Database(e.to_string()))?,
        refresh_token_encrypted: r.try_get("refresh_token_encrypted").unwrap_or(None),
        token_type: r.try_get("token_type").unwrap_or_else(|_| "Bearer".to_string()),
        scope: r.try_get("scope").unwrap_or(None),
        expires_at: r.try_get("expires_at").unwrap_or(None),
        is_revoked: is_revoked_int != 0,
        revoked_at,
        idp_session_id: r.try_get("idp_session_id").unwrap_or(None),
        created_at: DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    }))
}

pub async fn revoke_tokens_by_session_or_user(
    pool: &SqlitePool,
    user_id: &str,
    idp_session_id: Option<&str>,
) -> Result<u64> {
    let now_str = Utc::now().to_rfc3339();
    let query_str = if idp_session_id.is_some() {
        "UPDATE user_mcp_oauth_tokens SET is_revoked = 1, revoked_at = ? WHERE user_id = ? OR idp_session_id = ?"
    } else {
        "UPDATE user_mcp_oauth_tokens SET is_revoked = 1, revoked_at = ? WHERE user_id = ?"
    };

    let mut q = sqlx::query(query_str).bind(&now_str).bind(user_id);
    if let Some(sid) = idp_session_id {
        q = q.bind(sid);
    }

    let res = q.execute(pool).await.map_err(|e| LumisError::Database(e.to_string()))?;
    Ok(res.rows_affected())
}

pub async fn delete_user_mcp_oauth_token(
    pool: &SqlitePool,
    user_id: &str,
    server_id: &str,
) -> Result<bool> {
    let res = sqlx::query("DELETE FROM user_mcp_oauth_tokens WHERE user_id = ? AND server_id = ?")
        .bind(user_id)
        .bind(server_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}

pub async fn get_user_mcp_preference(
    pool: &SqlitePool,
    user_id: &str,
    server_id: &str,
) -> Result<Option<bool>> {
    let row = sqlx::query(
        "SELECT is_enabled FROM user_mcp_preferences WHERE user_id = ? AND server_id = ?",
    )
    .bind(user_id)
    .bind(server_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(row.map(|r| {
        let val: i64 = r.try_get("is_enabled").unwrap_or(1);
        val != 0
    }))
}

pub async fn set_user_mcp_preference(
    pool: &SqlitePool,
    user_id: &str,
    server_id: &str,
    is_enabled: bool,
) -> Result<()> {
    let is_enabled_int = if is_enabled { 1 } else { 0 };
    let updated_at_str = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO user_mcp_preferences (user_id, server_id, is_enabled, updated_at)
        VALUES (?, ?, ?, ?)
        ON CONFLICT (user_id, server_id) DO UPDATE SET
            is_enabled = excluded.is_enabled,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(user_id)
    .bind(server_id)
    .bind(is_enabled_int)
    .bind(updated_at_str)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(())
}

pub async fn list_user_mcp_preferences(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<UserMcpPreferenceRecord>> {
    let rows = sqlx::query(
        "SELECT user_id, server_id, is_enabled, updated_at FROM user_mcp_preferences WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        let is_enabled_int: i64 = r.try_get("is_enabled").unwrap_or(1);
        let updated_at_str: String = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;
        result.push(UserMcpPreferenceRecord {
            user_id: r.try_get("user_id").map_err(|e| LumisError::Database(e.to_string()))?,
            server_id: r.try_get("server_id").map_err(|e| LumisError::Database(e.to_string()))?,
            is_enabled: is_enabled_int != 0,
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        });
    }

    Ok(result)
}
