#![deny(unsafe_code)]

use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use sqlx::{PgPool, Row};

pub async fn check_and_increment_mcp_quota(
    pool: &PgPool,
    user_id: &str,
    server_id: &str,
    daily_limit: i32,
) -> Result<bool> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let now = Utc::now();

    let row = sqlx::query(
        "SELECT request_count FROM mcp_usage_quotas WHERE user_id = $1 AND server_id = $2 AND window_date = $3",
    )
    .bind(user_id)
    .bind(server_id)
    .bind(&today)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let current_count: i32 = row.and_then(|r| r.try_get("request_count").ok()).unwrap_or(0);

    if current_count >= daily_limit {
        return Ok(false);
    }

    let quota_id = format!("qta-{}-{}", user_id, today);
    sqlx::query(
        r#"
        INSERT INTO mcp_usage_quotas (id, user_id, server_id, window_date, request_count, updated_at)
        VALUES ($1, $2, $3, $4, 1, $5)
        ON CONFLICT (user_id, server_id, window_date) DO UPDATE SET
            request_count = mcp_usage_quotas.request_count + 1,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(&quota_id)
    .bind(user_id)
    .bind(server_id)
    .bind(&today)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(true)
}

pub async fn get_mcp_quota_usage(
    pool: &PgPool,
    user_id: &str,
    server_id: &str,
) -> Result<i32> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let row = sqlx::query(
        "SELECT request_count FROM mcp_usage_quotas WHERE user_id = $1 AND server_id = $2 AND window_date = $3",
    )
    .bind(user_id)
    .bind(server_id)
    .bind(&today)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let count: i32 = row.and_then(|r| r.try_get("request_count").ok()).unwrap_or(0);
    Ok(count)
}
