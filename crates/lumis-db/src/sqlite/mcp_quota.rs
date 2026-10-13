#![deny(unsafe_code)]

use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use sqlx::{Row, SqlitePool};

pub async fn check_and_increment_mcp_quota(
    pool: &SqlitePool,
    user_id: &str,
    server_id: &str,
    daily_limit: i32,
) -> Result<bool> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let updated_at_str = Utc::now().to_rfc3339();

    let row = sqlx::query(
        "SELECT request_count FROM mcp_usage_quotas WHERE user_id = ? AND server_id = ? AND window_date = ?",
    )
    .bind(user_id)
    .bind(server_id)
    .bind(&today)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let current_count: i64 = row.and_then(|r| r.try_get("request_count").ok()).unwrap_or(0);

    if current_count >= daily_limit as i64 {
        return Ok(false);
    }

    let quota_id = format!("qta-{}-{}", user_id, today);
    sqlx::query(
        r#"
        INSERT INTO mcp_usage_quotas (id, user_id, server_id, window_date, request_count, updated_at)
        VALUES (?, ?, ?, ?, 1, ?)
        ON CONFLICT (user_id, server_id, window_date) DO UPDATE SET
            request_count = mcp_usage_quotas.request_count + 1,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&quota_id)
    .bind(user_id)
    .bind(server_id)
    .bind(&today)
    .bind(updated_at_str)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(true)
}

pub async fn get_mcp_quota_usage(
    pool: &SqlitePool,
    user_id: &str,
    server_id: &str,
) -> Result<i32> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let row = sqlx::query(
        "SELECT request_count FROM mcp_usage_quotas WHERE user_id = ? AND server_id = ? AND window_date = ?",
    )
    .bind(user_id)
    .bind(server_id)
    .bind(&today)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let count: i64 = row.and_then(|r| r.try_get("request_count").ok()).unwrap_or(0);
    Ok(count as i32)
}
