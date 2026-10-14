use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{CreateTaskInput, SessionTask, TaskStatus, UpdateTaskInput};
use sqlx::{PgPool, Row};
use std::str::FromStr;
use uuid::Uuid;

pub async fn create_session_task(
    pool: &PgPool,
    input: &CreateTaskInput,
) -> Result<SessionTask> {
    let id = input
        .id
        .clone()
        .unwrap_or_else(|| format!("task-{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()));
    let now = Utc::now();
    let order_index = match input.order_index {
        Some(idx) => idx,
        None => {
            let max_order: Option<i32> = sqlx::query_scalar(
                "SELECT MAX(order_index) FROM session_tasks WHERE session_id = $1",
            )
            .bind(&input.session_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| LumisError::Database(e.to_string()))?
            .flatten();
            max_order.map(|m| m + 1).unwrap_or(0)
        }
    };

    sqlx::query(
        r#"
        INSERT INTO session_tasks (
            id, session_id, parent_task_id, title, description,
            status, order_index, result_summary, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, $8, $9)
        "#,
    )
    .bind(&id)
    .bind(&input.session_id)
    .bind(&input.parent_task_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.status.as_str())
    .bind(order_index)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(SessionTask {
        id,
        session_id: input.session_id.clone(),
        parent_task_id: input.parent_task_id.clone(),
        title: input.title.clone(),
        description: input.description.clone(),
        status: input.status,
        order_index,
        result_summary: None,
        created_at: now,
        updated_at: now,
    })
}

pub async fn get_session_task(pool: &PgPool, task_id: &str) -> Result<Option<SessionTask>> {
    let row = sqlx::query(
        r#"
        SELECT id, session_id, parent_task_id, title, description,
               status, order_index, result_summary, created_at, updated_at
        FROM session_tasks WHERE id = $1
        "#,
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(row.map(parse_pg_task))
}

pub async fn update_session_task(
    pool: &PgPool,
    task_id: &str,
    update: &UpdateTaskInput,
) -> Result<Option<SessionTask>> {
    let existing = get_session_task(pool, task_id).await?;
    let Some(mut task) = existing else {
        return Ok(None);
    };

    if let Some(ref t) = update.title {
        task.title = t.clone();
    }
    if let Some(ref d) = update.description {
        task.description = Some(d.clone());
    }
    if let Some(s) = update.status {
        task.status = s;
    }
    if let Some(idx) = update.order_index {
        task.order_index = idx;
    }
    if let Some(ref res) = update.result_summary {
        task.result_summary = Some(res.clone());
    }
    task.updated_at = Utc::now();

    sqlx::query(
        r#"
        UPDATE session_tasks
        SET title = $1, description = $2, status = $3, order_index = $4,
            result_summary = $5, updated_at = $6
        WHERE id = $7
        "#,
    )
    .bind(&task.title)
    .bind(&task.description)
    .bind(task.status.as_str())
    .bind(task.order_index)
    .bind(&task.result_summary)
    .bind(task.updated_at)
    .bind(task_id)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(Some(task))
}

pub async fn list_session_tasks(
    pool: &PgPool,
    session_id: &str,
) -> Result<Vec<SessionTask>> {
    let rows = sqlx::query(
        r#"
        SELECT id, session_id, parent_task_id, title, description,
               status, order_index, result_summary, created_at, updated_at
        FROM session_tasks
        WHERE session_id = $1
        ORDER BY order_index ASC, created_at ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(rows.into_iter().map(parse_pg_task).collect())
}

pub async fn delete_session_task(pool: &PgPool, task_id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM session_tasks WHERE id = $1")
        .bind(task_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}

pub async fn clear_session_tasks(pool: &PgPool, session_id: &str) -> Result<u64> {
    let res = sqlx::query("DELETE FROM session_tasks WHERE session_id = $1")
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected())
}

fn parse_pg_task(row: sqlx::postgres::PgRow) -> SessionTask {
    let status_str: String = row.get("status");
    let status = TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Pending);

    SessionTask {
        id: row.get("id"),
        session_id: row.get("session_id"),
        parent_task_id: row.get("parent_task_id"),
        title: row.get("title"),
        description: row.get("description"),
        status,
        order_index: row.get("order_index"),
        result_summary: row.get("result_summary"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
