use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::UserIdentity;
use sqlx::{Row, SqlitePool};

pub async fn find_identity(
    pool: &SqlitePool,
    provider: &str,
    provider_subject_id: &str,
) -> Result<Option<UserIdentity>> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, provider, provider_subject_id, credential_data, last_login_at, created_at, updated_at
        FROM user_identities
        WHERE provider = ? AND provider_subject_id = ?
        "#,
    )
    .bind(provider)
    .bind(provider_subject_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    match row {
        Some(r) => {
            let cred_raw: Option<String> = r.try_get("credential_data").unwrap_or(None);
            let cred_json = cred_raw.and_then(|s| serde_json::from_str(&s).ok());
            let last_login_str: Option<String> = r.try_get("last_login_at").unwrap_or(None);
            let created_at_str: String = r.try_get("created_at").map_err(|e| LumisError::Database(e.to_string()))?;
            let updated_at_str: String = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;

            Ok(Some(UserIdentity {
                id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
                user_id: r.try_get("user_id").map_err(|e| LumisError::Database(e.to_string()))?,
                provider: r.try_get("provider").map_err(|e| LumisError::Database(e.to_string()))?,
                provider_subject_id: r.try_get("provider_subject_id").map_err(|e| LumisError::Database(e.to_string()))?,
                credential_data: cred_json,
                last_login_at: last_login_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()).map(|dt| dt.with_timezone(&Utc)),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }))
        }
        None => Ok(None),
    }
}

pub async fn link_identity(pool: &SqlitePool, identity: &UserIdentity) -> Result<UserIdentity> {
    let cred_str = identity.credential_data.as_ref().map(|v| v.to_string());
    let last_login_str = identity.last_login_at.map(|dt| dt.to_rfc3339());
    let created_at_str = identity.created_at.to_rfc3339();
    let updated_at_str = identity.updated_at.to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO user_identities (id, user_id, provider, provider_subject_id, credential_data, last_login_at, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(provider, provider_subject_id) DO UPDATE SET
            user_id = excluded.user_id,
            credential_data = COALESCE(excluded.credential_data, user_identities.credential_data),
            last_login_at = excluded.last_login_at,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&identity.id)
    .bind(&identity.user_id)
    .bind(&identity.provider)
    .bind(&identity.provider_subject_id)
    .bind(&cred_str)
    .bind(&last_login_str)
    .bind(&created_at_str)
    .bind(&updated_at_str)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(identity.clone())
}

pub async fn list_identities_by_user(pool: &SqlitePool, user_id: &str) -> Result<Vec<UserIdentity>> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, provider, provider_subject_id, credential_data, last_login_at, created_at, updated_at
        FROM user_identities
        WHERE user_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut result = Vec::new();
    for r in rows {
        let cred_raw: Option<String> = r.try_get("credential_data").unwrap_or(None);
        let cred_json = cred_raw.and_then(|s| serde_json::from_str(&s).ok());
        let last_login_str: Option<String> = r.try_get("last_login_at").unwrap_or(None);
        let created_at_str: String = r.try_get("created_at").map_err(|e| LumisError::Database(e.to_string()))?;
        let updated_at_str: String = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;

        result.push(UserIdentity {
            id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
            user_id: r.try_get("user_id").map_err(|e| LumisError::Database(e.to_string()))?,
            provider: r.try_get("provider").map_err(|e| LumisError::Database(e.to_string()))?,
            provider_subject_id: r.try_get("provider_subject_id").map_err(|e| LumisError::Database(e.to_string()))?,
            credential_data: cred_json,
            last_login_at: last_login_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()).map(|dt| dt.with_timezone(&Utc)),
            created_at: DateTime::parse_from_rfc3339(&created_at_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
        });
    }

    Ok(result)
}

pub async fn unlink_identity(pool: &SqlitePool, user_id: &str, provider: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM user_identities WHERE user_id = ? AND provider = ?")
        .bind(user_id)
        .bind(provider)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}
