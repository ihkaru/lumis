use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{CursorPage, CursorQuery, Message, MessageRole, NewMessage};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

pub async fn append_message(pool: &SqlitePool, new_message: &NewMessage) -> Result<Message> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let role_str = match new_message.role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
        MessageRole::Tool => "tool",
    };
    let meta_str = serde_json::to_string(
        &new_message
            .metadata
            .clone()
            .unwrap_or(serde_json::Value::Null),
    )
    .unwrap_or_else(|_| "{}".to_string());

    sqlx::query(
        "INSERT INTO messages (id, session_id, role, content, model_id, token_count, metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&new_message.session_id)
    .bind(role_str)
    .bind(&new_message.content)
    .bind(&new_message.model_id)
    .bind(new_message.token_count.map(|t| t as i64))
    .bind(&meta_str)
    .bind(now.to_rfc3339())
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
        metadata: new_message
            .metadata
            .clone()
            .unwrap_or(serde_json::Value::Null),
        created_at: now,
    })
}

pub async fn list_messages(
    pool: &SqlitePool,
    session_id: &str,
    query: &CursorQuery,
) -> Result<CursorPage<Message>> {
    let limit = query.limit.max(1).min(100);
    let fetch_limit = (limit + 1) as i64;

    let rows = if let (Some(c_time), Some(c_id)) = (query.cursor_time, &query.cursor_id) {
        sqlx::query(
            "SELECT id, session_id, role, content, model_id, token_count, metadata, created_at FROM messages WHERE session_id = ? AND (created_at < ? OR (created_at = ? AND id < ?)) ORDER BY created_at DESC, id DESC LIMIT ?"
        )
        .bind(session_id)
        .bind(c_time.to_rfc3339())
        .bind(c_time.to_rfc3339())
        .bind(c_id)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            "SELECT id, session_id, role, content, model_id, token_count, metadata, created_at FROM messages WHERE session_id = ? ORDER BY created_at DESC, id DESC LIMIT ?"
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
        let meta_s: String = r.get("metadata");
        let created_at_str: String = r.get("created_at");

        items.push(Message {
            id: r.get("id"),
            session_id: r.get("session_id"),
            role,
            content: r.get("content"),
            model_id: r.get("model_id"),
            token_count: r.get::<Option<i64>, _>("token_count").map(|t| t as u32),
            metadata: serde_json::from_str(&meta_s).unwrap_or(serde_json::Value::Null),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
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
    pool: &SqlitePool,
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
        WHERE s.user_id = ? AND m.content LIKE ?
        ORDER BY m.created_at DESC
        LIMIT ?
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
        let meta_s: String = r.get("metadata");
        let created_at_str: String = r.get("created_at");

        items.push(Message {
            id: r.get("id"),
            session_id: r.get("session_id"),
            role,
            content: r.get("content"),
            model_id: r.get("model_id"),
            token_count: r.get::<Option<i64>, _>("token_count").map(|t| t as u32),
            metadata: serde_json::from_str(&meta_s).unwrap_or(serde_json::Value::Null),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        });
    }

    Ok(items)
}
