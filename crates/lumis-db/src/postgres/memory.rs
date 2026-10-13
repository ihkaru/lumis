use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{NewUserMemory, UserMemory};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn list_user_memories(pool: &PgPool, user_id: &str) -> Result<Vec<UserMemory>> {
    let rows = sqlx::query(
        "SELECT id, user_id, category, content, created_at, updated_at FROM user_memories WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut items = Vec::new();
    for r in rows {
        items.push(UserMemory {
            id: r.get("id"),
            user_id: r.get("user_id"),
            category: r.get("category"),
            content: r.get("content"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        });
    }

    Ok(items)
}

pub async fn add_user_memory(pool: &PgPool, new_mem: &NewUserMemory) -> Result<UserMemory> {
    let id = Uuid::new_v4().to_string();
    let category = new_mem.category.clone().unwrap_or_else(|| "preference".to_string());
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO user_memories (id, user_id, category, content, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&id)
    .bind(&new_mem.user_id)
    .bind(&category)
    .bind(&new_mem.content)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(UserMemory {
        id,
        user_id: new_mem.user_id.clone(),
        category,
        content: new_mem.content.clone(),
        created_at: now,
        updated_at: now,
    })
}

pub async fn delete_user_memory(pool: &PgPool, id: &str, user_id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM user_memories WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}
