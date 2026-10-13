use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{EnterpriseRole, User};
use sqlx::{Row, SqlitePool};

pub async fn get_user_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>> {
    let row = sqlx::query(
        "SELECT id, username, email, role, tenant_id, preferences, is_active, password_hash, created_at, updated_at FROM users WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(row.map(|r| {
        let role_num: i32 = r.get("role");
        let role = match role_num {
            5 => EnterpriseRole::SuperAdmin,
            4 => EnterpriseRole::TenantAdmin,
            3 => EnterpriseRole::PowerUser,
            1 => EnterpriseRole::Guest,
            _ => EnterpriseRole::Member,
        };
        let prefs_str: String = r.get("preferences");
        let created_at_str: String = r.get("created_at");
        let updated_at_str: String = r.get("updated_at");

        User {
            id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
            role,
            tenant_id: r.get("tenant_id"),
            preferences: serde_json::from_str(&prefs_str).unwrap_or(serde_json::Value::Null),
            is_active: r.get::<i32, _>("is_active") == 1,
            password_hash: r.get("password_hash"),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }
    }))
}

pub async fn upsert_user(pool: &SqlitePool, user: &User) -> Result<User> {
    let now = Utc::now().to_rfc3339();
    let prefs = serde_json::to_string(&user.preferences).unwrap_or_else(|_| "{}".to_string());

    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, role, tenant_id, preferences, is_active, password_hash, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            email = excluded.email,
            role = excluded.role,
            preferences = excluded.preferences,
            is_active = excluded.is_active,
            password_hash = COALESCE(excluded.password_hash, users.password_hash),
            updated_at = excluded.updated_at
        "#
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.email)
    .bind(user.role.level() as i32)
    .bind(&user.tenant_id)
    .bind(&prefs)
    .bind(if user.is_active { 1 } else { 0 })
    .bind(&user.password_hash)
    .bind(user.created_at.to_rfc3339())
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(user.clone())
}

pub async fn list_users(pool: &SqlitePool, limit: usize) -> Result<Vec<User>> {
    let rows = sqlx::query(
        "SELECT id, username, email, role, tenant_id, preferences, is_active, password_hash, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT ?"
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut users = Vec::new();
    for r in rows {
        let role_num: i32 = r.get("role");
        let role = match role_num {
            5 => EnterpriseRole::SuperAdmin,
            4 => EnterpriseRole::TenantAdmin,
            3 => EnterpriseRole::PowerUser,
            1 => EnterpriseRole::Guest,
            _ => EnterpriseRole::Member,
        };
        let prefs_str: String = r.get("preferences");
        let created_at_str: String = r.get("created_at");
        let updated_at_str: String = r.get("updated_at");

        users.push(User {
            id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
            role,
            tenant_id: r.get("tenant_id"),
            preferences: serde_json::from_str(&prefs_str).unwrap_or(serde_json::Value::Null),
            is_active: r.get::<i32, _>("is_active") == 1,
            password_hash: r.get("password_hash"),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        });
    }
    Ok(users)
}

