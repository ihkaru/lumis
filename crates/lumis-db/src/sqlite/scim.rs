use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::scim::{ScimEmail, ScimGroup, ScimMemberRef, ScimUser, SCIM_GROUP_SCHEMA, SCIM_USER_SCHEMA};
use sqlx::{Row, SqlitePool};

pub async fn list_scim_users(pool: &SqlitePool) -> Result<Vec<ScimUser>> {
    let rows = sqlx::query(
        "SELECT id, external_id, username, display_name, active, emails FROM scim_users ORDER BY created_at ASC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut users = Vec::new();
    for r in rows {
        let emails_raw: String = r.try_get("emails").unwrap_or_else(|_| "[]".to_string());
        let emails: Vec<ScimEmail> = serde_json::from_str(&emails_raw).unwrap_or_default();
        let active_int: i64 = r.try_get("active").unwrap_or(1);

        users.push(ScimUser {
            schemas: vec![SCIM_USER_SCHEMA.to_string()],
            id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
            user_name: r.try_get("username").map_err(|e| LumisError::Database(e.to_string()))?,
            display_name: r.try_get("display_name").unwrap_or(None),
            emails,
            active: active_int == 1,
            roles: vec![],
        });
    }

    Ok(users)
}

pub async fn get_scim_user(pool: &SqlitePool, id: &str) -> Result<Option<ScimUser>> {
    let row = sqlx::query(
        "SELECT id, external_id, username, display_name, active, emails FROM scim_users WHERE id = ? OR external_id = ?"
    )
    .bind(id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    match row {
        Some(r) => {
            let emails_raw: String = r.try_get("emails").unwrap_or_else(|_| "[]".to_string());
            let emails: Vec<ScimEmail> = serde_json::from_str(&emails_raw).unwrap_or_default();
            let active_int: i64 = r.try_get("active").unwrap_or(1);

            Ok(Some(ScimUser {
                schemas: vec![SCIM_USER_SCHEMA.to_string()],
                id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
                user_name: r.try_get("username").map_err(|e| LumisError::Database(e.to_string()))?,
                display_name: r.try_get("display_name").unwrap_or(None),
                emails,
                active: active_int == 1,
                roles: vec![],
            }))
        }
        None => Ok(None),
    }
}

pub async fn upsert_scim_user(pool: &SqlitePool, user: &ScimUser) -> Result<ScimUser> {
    let emails_json = serde_json::to_string(&user.emails).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();
    let active_int = if user.active { 1 } else { 0 };

    sqlx::query(
        r#"
        INSERT INTO scim_users (id, external_id, username, display_name, active, emails, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            username = excluded.username,
            display_name = excluded.display_name,
            active = excluded.active,
            emails = excluded.emails,
            updated_at = excluded.updated_at
        "#
    )
    .bind(&user.id)
    .bind(&user.id)
    .bind(&user.user_name)
    .bind(&user.display_name)
    .bind(active_int)
    .bind(&emails_json)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(user.clone())
}

pub async fn update_scim_user_status(pool: &SqlitePool, id: &str, active: bool) -> Result<bool> {
    let now = Utc::now().to_rfc3339();
    let active_int = if active { 1 } else { 0 };

    let res = sqlx::query("UPDATE scim_users SET active = ?, updated_at = ? WHERE id = ?")
        .bind(active_int)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}

pub async fn delete_scim_user(pool: &SqlitePool, id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM scim_users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}

pub async fn list_scim_groups(pool: &SqlitePool) -> Result<Vec<ScimGroup>> {
    let rows = sqlx::query("SELECT id, display_name, members FROM scim_groups ORDER BY created_at ASC")
        .fetch_all(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut groups = Vec::new();
    for r in rows {
        let mem_raw: String = r.try_get("members").unwrap_or_else(|_| "[]".to_string());
        let members: Vec<ScimMemberRef> = serde_json::from_str(&mem_raw).unwrap_or_default();

        groups.push(ScimGroup {
            schemas: vec![SCIM_GROUP_SCHEMA.to_string()],
            id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
            display_name: r.try_get("display_name").map_err(|e| LumisError::Database(e.to_string()))?,
            members,
        });
    }

    Ok(groups)
}

pub async fn get_scim_group(pool: &SqlitePool, id: &str) -> Result<Option<ScimGroup>> {
    let row = sqlx::query("SELECT id, display_name, members FROM scim_groups WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    match row {
        Some(r) => {
            let mem_raw: String = r.try_get("members").unwrap_or_else(|_| "[]".to_string());
            let members: Vec<ScimMemberRef> = serde_json::from_str(&mem_raw).unwrap_or_default();

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

pub async fn upsert_scim_group(pool: &SqlitePool, group: &ScimGroup) -> Result<ScimGroup> {
    let mem_json = serde_json::to_string(&group.members).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO scim_groups (id, display_name, members, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            display_name = excluded.display_name,
            members = excluded.members,
            updated_at = excluded.updated_at
        "#
    )
    .bind(&group.id)
    .bind(&group.display_name)
    .bind(&mem_json)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(group.clone())
}

pub async fn delete_scim_group(pool: &SqlitePool, id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM scim_groups WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}
