use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{CursorPage, CursorQuery, Message, MessageRole, NewMessage};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn append_message(pool: &PgPool, new_message: &NewMessage) -> Result<Message> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let role_str = match new_message.role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
        MessageRole::Tool => "tool",
    };
    let meta = new_message
        .metadata
        .clone()
        .unwrap_or(serde_json::Value::Null);

    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, model_id, token_count, metadata, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(&id)
    .bind(&new_message.session_id)
    .bind(role_str)
    .bind(&new_message.content)
    .bind(&new_message.model_id)
    .bind(new_message.token_count.map(|t| t as i32))
    .bind(&meta)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(Message {
        id,
        session_id: new_message.session_id.clone(),
        role: new_message.role,
        content: new_message.content.clone(),
        model_id: new_message.model_id.clone(),
        token_count: new_message.token_count,
        metadata: meta,
        created_at: now,
    })
}

pub async fn list_messages(
    pool: &PgPool,
    session_id: &str,
    query: &CursorQuery,
) -> Result<CursorPage<Message>> {
    let limit = query.limit.max(1).min(100);
    let fetch_limit = (limit + 1) as i64;

    let rows = if let (Some(c_time), Some(c_id)) = (query.cursor_time, &query.cursor_id) {
        sqlx::query(
            "SELECT id, session_id, role, content, model_id, token_count, metadata, created_at FROM messages WHERE session_id = $1 AND (created_at, id) < ($2, $3) ORDER BY created_at DESC, id DESC LIMIT $4"
        )
        .bind(session_id)
        .bind(c_time)
        .bind(c_id)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            "SELECT id, session_id, role, content, model_id, token_count, metadata, created_at FROM messages WHERE session_id = $1 ORDER BY created_at DESC, id DESC LIMIT $2"
        )
        .bind(session_id)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut items = Vec::new();
    for r in rows {
        let role_s: String = r.get("role");
        let role = match role_s.as_str() {
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            "tool" => MessageRole::Tool,
            _ => MessageRole::User,
        };

        items.push(Message {
            id: r.get("id"),
            session_id: r.get("session_id"),
            role,
            content: r.get("content"),
            model_id: r.get("model_id"),
            token_count: r.get::<Option<i32>, _>("token_count").map(|t| t as u32),
            metadata: r.get("metadata"),
            created_at: r.get("created_at"),
        });
    }

    let has_more = items.len() > limit;
    if has_more {
        items.truncate(limit);
    }

    let next_cursor = items.last().map(|m| (m.created_at, m.id.clone()));

    Ok(CursorPage {
        items,
        next_cursor_time: next_cursor.as_ref().map(|c| c.0),
        next_cursor_id: next_cursor.map(|c| c.1),
        has_more,
    })
}

pub async fn search_messages(
    pool: &PgPool,
    user_id: &str,
    query_text: &str,
    limit: usize,
) -> Result<Vec<Message>> {
    let pattern = format!("%{}%", query_text);
    let rows = sqlx::query(
        r#"
        SELECT m.id, m.session_id, m.role, m.content, m.model_id, m.token_count, m.metadata, m.created_at
        FROM messages m
        JOIN sessions s ON s.id = m.session_id
        WHERE s.user_id = $1 AND m.content ILIKE $2
        ORDER BY m.created_at DESC
        LIMIT $3
        "#
    )
    .bind(user_id)
    .bind(&pattern)
    .bind(limit as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut items = Vec::new();
    for r in rows {
        let role_s: String = r.get("role");
        let role = match role_s.as_str() {
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            "tool" => MessageRole::Tool,
            _ => MessageRole::User,
        };

        items.push(Message {
            id: r.get("id"),
            session_id: r.get("session_id"),
            role,
            content: r.get("content"),
            model_id: r.get("model_id"),
            token_count: r.get::<Option<i32>, _>("token_count").map(|t| t as u32),
            metadata: r.get("metadata"),
            created_at: r.get("created_at"),
        });
    }

    Ok(items)
}
