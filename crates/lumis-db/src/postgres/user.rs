use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{EnterpriseRole, User};
use sqlx::{PgPool, Row};

pub async fn get_user_by_id(pool: &PgPool, id: &str) -> Result<Option<User>> {
    let row = sqlx::query(
        "SELECT id, username, email, role, tenant_id, preferences, is_active, password_hash, created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(row.map(|r| {
        let role_num: i16 = r.get("role");
        let role = match role_num {
            5 => EnterpriseRole::SuperAdmin,
            4 => EnterpriseRole::TenantAdmin,
            3 => EnterpriseRole::PowerUser,
            1 => EnterpriseRole::Guest,
            _ => EnterpriseRole::Member,
        };
        User {
            id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
            role,
            tenant_id: r.get("tenant_id"),
            preferences: r.get("preferences"),
            is_active: r.get("is_active"),
            password_hash: r.get("password_hash"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }
    }))
}

pub async fn upsert_user(pool: &PgPool, user: &User) -> Result<User> {
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, role, tenant_id, preferences, is_active, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT(id) DO UPDATE SET
            email = EXCLUDED.email,
            role = EXCLUDED.role,
            preferences = EXCLUDED.preferences,
            is_active = EXCLUDED.is_active,
            password_hash = COALESCE(EXCLUDED.password_hash, users.password_hash),
            updated_at = EXCLUDED.updated_at
        "#
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.email)
    .bind(user.role.level() as i16)
    .bind(&user.tenant_id)
    .bind(&user.preferences)
    .bind(user.is_active)
    .bind(&user.password_hash)
    .bind(user.created_at)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(user.clone())
}

pub async fn list_users(pool: &PgPool, limit: usize) -> Result<Vec<User>> {
    let rows = sqlx::query(
        "SELECT id, username, email, role, tenant_id, preferences, is_active, password_hash, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1"
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut users = Vec::new();
    for r in rows {
        let role_num: i16 = r.get("role");
        let role = match role_num {
            5 => EnterpriseRole::SuperAdmin,
            4 => EnterpriseRole::TenantAdmin,
            3 => EnterpriseRole::PowerUser,
            1 => EnterpriseRole::Guest,
            _ => EnterpriseRole::Member,
        };

        users.push(User {
            id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
            role,
            tenant_id: r.get("tenant_id"),
            preferences: r.get("preferences"),
            is_active: r.get("is_active"),
            password_hash: r.get("password_hash"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        });
    }
    Ok(users)
}

