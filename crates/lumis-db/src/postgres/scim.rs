use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::scim::{ScimEmail, ScimGroup, ScimMemberRef, ScimUser, SCIM_GROUP_SCHEMA, SCIM_USER_SCHEMA};
use sqlx::{PgPool, Row};

pub async fn list_scim_users(pool: &PgPool) -> Result<Vec<ScimUser>> {
    let rows = sqlx::query(
        "SELECT id, external_id, username, display_name, active, emails FROM scim_users ORDER BY created_at ASC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut users = Vec::new();
    for r in rows {
        let emails_val: Option<serde_json::Value> = r.try_get("emails").unwrap_or(None);
        let emails: Vec<ScimEmail> = emails_val.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default();
        let active: bool = r.try_get("active").unwrap_or(true);

        users.push(ScimUser {
            schemas: vec![SCIM_USER_SCHEMA.to_string()],
            id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
            user_name: r.try_get("username").map_err(|e| LumisError::Database(e.to_string()))?,
            display_name: r.try_get("display_name").unwrap_or(None),
            emails,
            active,
            roles: vec![],
        });
    }

    Ok(users)
}

pub async fn get_scim_user(pool: &PgPool, id: &str) -> Result<Option<ScimUser>> {
    let row = sqlx::query(
        "SELECT id, external_id, username, display_name, active, emails FROM scim_users WHERE id = $1 OR external_id = $2"
    )
    .bind(id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    match row {
        Some(r) => {
            let emails_val: Option<serde_json::Value> = r.try_get("emails").unwrap_or(None);
            let emails: Vec<ScimEmail> = emails_val.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default();
            let active: bool = r.try_get("active").unwrap_or(true);

            Ok(Some(ScimUser {
                schemas: vec![SCIM_USER_SCHEMA.to_string()],
                id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
                user_name: r.try_get("username").map_err(|e| LumisError::Database(e.to_string()))?,
                display_name: r.try_get("display_name").unwrap_or(None),
                emails,
                active,
                roles: vec![],
            }))
        }
        None => Ok(None),
    }
}

pub async fn upsert_scim_user(pool: &PgPool, user: &ScimUser) -> Result<ScimUser> {
    let emails_val = serde_json::to_value(&user.emails).unwrap_or(serde_json::json!([]));
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO scim_users (id, external_id, username, display_name, active, emails, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT(id) DO UPDATE SET
            username = EXCLUDED.username,
            display_name = EXCLUDED.display_name,
            active = EXCLUDED.active,
            emails = EXCLUDED.emails,
            updated_at = EXCLUDED.updated_at
        "#
    )
    .bind(&user.id)
    .bind(&user.id)
    .bind(&user.user_name)
    .bind(&user.display_name)
    .bind(user.active)
    .bind(emails_val)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(user.clone())
}

pub async fn update_scim_user_status(pool: &PgPool, id: &str, active: bool) -> Result<bool> {
    let now = Utc::now();

    let res = sqlx::query("UPDATE scim_users SET active = $1, updated_at = $2 WHERE id = $3")
        .bind(active)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}

pub async fn delete_scim_user(pool: &PgPool, id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM scim_users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}

pub async fn list_scim_groups(pool: &PgPool) -> Result<Vec<ScimGroup>> {
    let rows = sqlx::query("SELECT id, display_name, members FROM scim_groups ORDER BY created_at ASC")
        .fetch_all(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut groups = Vec::new();
    for r in rows {
        let mem_val: Option<serde_json::Value> = r.try_get("members").unwrap_or(None);
        let members: Vec<ScimMemberRef> = mem_val.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default();

        groups.push(ScimGroup {
            schemas: vec![SCIM_GROUP_SCHEMA.to_string()],
            id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
            display_name: r.try_get("display_name").map_err(|e| LumisError::Database(e.to_string()))?,
            members,
        });
    }

    Ok(groups)
}

pub async fn get_scim_group(pool: &PgPool, id: &str) -> Result<Option<ScimGroup>> {
    let row = sqlx::query("SELECT id, display_name, members FROM scim_groups WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    match row {
        Some(r) => {
            let mem_val: Option<serde_json::Value> = r.try_get("members").unwrap_or(None);
            let members: Vec<ScimMemberRef> = mem_val.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default();

            Ok(Some(ScimGroup {
                schemas: vec![SCIM_GROUP_SCHEMA.to_string()],
                id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
                display_name: r.try_get("display_name").map_err(|e| LumisError::Database(e.to_string()))?,
                members,
            }))
        }
        None => Ok(None),
    }
}

pub async fn upsert_scim_group(pool: &PgPool, group: &ScimGroup) -> Result<ScimGroup> {
    let mem_val = serde_json::to_value(&group.members).unwrap_or(serde_json::json!([]));
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO scim_groups (id, display_name, members, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT(id) DO UPDATE SET
            display_name = EXCLUDED.display_name,
            members = EXCLUDED.members,
            updated_at = EXCLUDED.updated_at
        "#
    )
    .bind(&group.id)
    .bind(&group.display_name)
    .bind(mem_val)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(group.clone())
}

pub async fn delete_scim_group(pool: &PgPool, id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM scim_groups WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}
