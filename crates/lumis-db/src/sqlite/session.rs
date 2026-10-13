use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{CursorPage, CursorQuery, NewSession, Session};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

pub async fn create_session(pool: &SqlitePool, new_session: &NewSession) -> Result<Session> {
    let id = new_session.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO sessions (id, user_id, title, model_id, system_prompt, is_pinned, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 0, ?, ?)"
    )
    .bind(&id)
    .bind(&new_session.user_id)
    .bind(&new_session.title)
    .bind(&new_session.model_id)
    .bind(&new_session.system_prompt)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(Session {
        id,
        user_id: new_session.user_id.clone(),
        title: new_session.title.clone(),
        model_id: new_session.model_id.clone(),
        system_prompt: new_session.system_prompt.clone(),
        is_pinned: false,
        created_at: now,
        updated_at: now,
    })
}

pub async fn get_session(pool: &SqlitePool, id: &str) -> Result<Option<Session>> {
    let row = sqlx::query(
        "SELECT id, user_id, title, model_id, system_prompt, is_pinned, created_at, updated_at FROM sessions WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(row.map(|r| {
        let created_at_str: String = r.get("created_at");
        let updated_at_str: String = r.get("updated_at");
        Session {
            id: r.get("id"),
            user_id: r.get("user_id"),
            title: r.get("title"),
            model_id: r.get("model_id"),
            system_prompt: r.get("system_prompt"),
            is_pinned: r.get::<i32, _>("is_pinned") == 1,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }
    }))
}

pub async fn list_sessions(
    pool: &SqlitePool,
    user_id: &str,
    query: &CursorQuery,
) -> Result<CursorPage<Session>> {
    let limit = query.limit.max(1).min(100);
    let fetch_limit = (limit + 1) as i64;

    let rows = if let (Some(c_time), Some(c_id)) = (query.cursor_time, &query.cursor_id) {
        sqlx::query(
            "SELECT id, user_id, title, model_id, system_prompt, is_pinned, created_at, updated_at FROM sessions WHERE user_id = ? AND (created_at < ? OR (created_at = ? AND id < ?)) ORDER BY created_at DESC, id DESC LIMIT ?"
        )
        .bind(user_id)
        .bind(c_time.to_rfc3339())
        .bind(c_time.to_rfc3339())
        .bind(c_id)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            "SELECT id, user_id, title, model_id, system_prompt, is_pinned, created_at, updated_at FROM sessions WHERE user_id = ? ORDER BY created_at DESC, id DESC LIMIT ?"
        )
        .bind(user_id)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut items = Vec::new();
    for r in rows {
        let created_at_str: String = r.get("created_at");
        let updated_at_str: String = r.get("updated_at");
        items.push(Session {
            id: r.get("id"),
            user_id: r.get("user_id"),
            title: r.get("title"),
            model_id: r.get("model_id"),
            system_prompt: r.get("system_prompt"),
            is_pinned: r.get::<i32, _>("is_pinned") == 1,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        });
    }

    let has_more = items.len() > limit;
    if has_more {
        items.truncate(limit);
    }

    let next_cursor = items.last().map(|s| (s.created_at, s.id.clone()));

    Ok(CursorPage {
        items,
        next_cursor_time: next_cursor.as_ref().map(|c| c.0),
        next_cursor_id: next_cursor.map(|c| c.1),
        has_more,
    })
}

pub async fn delete_session(pool: &SqlitePool, id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;
    Ok(res.rows_affected() > 0)
}
