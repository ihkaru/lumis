use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{NewUserMemory, UserMemory};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

pub async fn list_user_memories(pool: &SqlitePool, user_id: &str) -> Result<Vec<UserMemory>> {
    let rows = sqlx::query(
        "SELECT id, user_id, category, content, created_at, updated_at FROM user_memories WHERE user_id = ? ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut items = Vec::new();
    for r in rows {
        let created_at_str: String = r.get("created_at");
        let updated_at_str: String = r.get("updated_at");
        items.push(UserMemory {
            id: r.get("id"),
            user_id: r.get("user_id"),
            category: r.get("category"),
            content: r.get("content"),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        });
    }

    Ok(items)
}

pub async fn add_user_memory(pool: &SqlitePool, new_mem: &NewUserMemory) -> Result<UserMemory> {
    let id = Uuid::new_v4().to_string();
    let category = new_mem.category.clone().unwrap_or_else(|| "preference".to_string());
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO user_memories (id, user_id, category, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&new_mem.user_id)
    .bind(&category)
    .bind(&new_mem.content)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
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

pub async fn delete_user_memory(pool: &SqlitePool, id: &str, user_id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM user_memories WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}
