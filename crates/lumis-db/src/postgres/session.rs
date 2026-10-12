use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{CursorPage, CursorQuery, NewSession, Session};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn create_session(pool: &PgPool, new_session: &NewSession) -> Result<Session> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO sessions (id, user_id, title, model_id, system_prompt, is_pinned, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, false, $6, $7)"
    )
    .bind(&id)
    .bind(&new_session.user_id)
    .bind(&new_session.title)
    .bind(&new_session.model_id)
    .bind(&new_session.system_prompt)
    .bind(now)
    .bind(now)
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

pub async fn get_session(pool: &PgPool, id: &str) -> Result<Option<Session>> {
    let row = sqlx::query(
        "SELECT id, user_id, title, model_id, system_prompt, is_pinned, created_at, updated_at FROM sessions WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(row.map(|r| Session {
        id: r.get("id"),
        user_id: r.get("user_id"),
        title: r.get("title"),
        model_id: r.get("model_id"),
        system_prompt: r.get("system_prompt"),
        is_pinned: r.get("is_pinned"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

pub async fn list_sessions(
    pool: &PgPool,
    user_id: &str,
    query: &CursorQuery,
) -> Result<CursorPage<Session>> {
    let limit = query.limit.max(1).min(100);
    let fetch_limit = (limit + 1) as i64;

    let rows = if let (Some(c_time), Some(c_id)) = (query.cursor_time, &query.cursor_id) {
        sqlx::query(
            "SELECT id, user_id, title, model_id, system_prompt, is_pinned, created_at, updated_at FROM sessions WHERE user_id = $1 AND (created_at, id) < ($2, $3) ORDER BY created_at DESC, id DESC LIMIT $4"
        )
        .bind(user_id)
        .bind(c_time)
        .bind(c_id)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            "SELECT id, user_id, title, model_id, system_prompt, is_pinned, created_at, updated_at FROM sessions WHERE user_id = $1 ORDER BY created_at DESC, id DESC LIMIT $2"
        )
        .bind(user_id)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut items = Vec::new();
    for r in rows {
        items.push(Session {
            id: r.get("id"),
            user_id: r.get("user_id"),
            title: r.get("title"),
            model_id: r.get("model_id"),
            system_prompt: r.get("system_prompt"),
            is_pinned: r.get("is_pinned"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
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

pub async fn delete_session(pool: &PgPool, id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;
    Ok(res.rows_affected() > 0)
}
