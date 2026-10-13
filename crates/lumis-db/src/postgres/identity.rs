use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::UserIdentity;
use sqlx::{PgPool, Row};

pub async fn find_identity(
    pool: &PgPool,
    provider: &str,
    provider_subject_id: &str,
) -> Result<Option<UserIdentity>> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, provider, provider_subject_id, credential_data, last_login_at, created_at, updated_at
        FROM user_identities
        WHERE provider = $1 AND provider_subject_id = $2
        "#,
    )
    .bind(provider)
    .bind(provider_subject_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    match row {
        Some(r) => {
            let cred_raw: Option<serde_json::Value> = r.try_get("credential_data").unwrap_or(None);
            let last_login: Option<DateTime<Utc>> = r.try_get("last_login_at").unwrap_or(None);
            let created_at: DateTime<Utc> = r.try_get("created_at").map_err(|e| LumisError::Database(e.to_string()))?;
            let updated_at: DateTime<Utc> = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;

            Ok(Some(UserIdentity {
                id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
                user_id: r.try_get("user_id").map_err(|e| LumisError::Database(e.to_string()))?,
                provider: r.try_get("provider").map_err(|e| LumisError::Database(e.to_string()))?,
                provider_subject_id: r.try_get("provider_subject_id").map_err(|e| LumisError::Database(e.to_string()))?,
                credential_data: cred_raw,
                last_login_at: last_login,
                created_at,
                updated_at,
            }))
        }
        None => Ok(None),
    }
}

pub async fn link_identity(pool: &PgPool, identity: &UserIdentity) -> Result<UserIdentity> {
    sqlx::query(
        r#"
        INSERT INTO user_identities (id, user_id, provider, provider_subject_id, credential_data, last_login_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT(provider, provider_subject_id) DO UPDATE SET
            user_id = EXCLUDED.user_id,
            credential_data = COALESCE(EXCLUDED.credential_data, user_identities.credential_data),
            last_login_at = EXCLUDED.last_login_at,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(&identity.id)
    .bind(&identity.user_id)
    .bind(&identity.provider)
    .bind(&identity.provider_subject_id)
    .bind(&identity.credential_data)
    .bind(identity.last_login_at)
    .bind(identity.created_at)
    .bind(identity.updated_at)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(identity.clone())
}

pub async fn list_identities_by_user(pool: &PgPool, user_id: &str) -> Result<Vec<UserIdentity>> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, provider, provider_subject_id, credential_data, last_login_at, created_at, updated_at
        FROM user_identities
        WHERE user_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let mut result = Vec::new();
    for r in rows {
        let cred_raw: Option<serde_json::Value> = r.try_get("credential_data").unwrap_or(None);
        let last_login: Option<DateTime<Utc>> = r.try_get("last_login_at").unwrap_or(None);
        let created_at: DateTime<Utc> = r.try_get("created_at").map_err(|e| LumisError::Database(e.to_string()))?;
        let updated_at: DateTime<Utc> = r.try_get("updated_at").map_err(|e| LumisError::Database(e.to_string()))?;

        result.push(UserIdentity {
            id: r.try_get("id").map_err(|e| LumisError::Database(e.to_string()))?,
            user_id: r.try_get("user_id").map_err(|e| LumisError::Database(e.to_string()))?,
            provider: r.try_get("provider").map_err(|e| LumisError::Database(e.to_string()))?,
            provider_subject_id: r.try_get("provider_subject_id").map_err(|e| LumisError::Database(e.to_string()))?,
            credential_data: cred_raw,
            last_login_at: last_login,
            created_at,
            updated_at,
        });
    }

    Ok(result)
}

pub async fn unlink_identity(pool: &PgPool, user_id: &str, provider: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM user_identities WHERE user_id = $1 AND provider = $2")
        .bind(user_id)
        .bind(provider)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(res.rows_affected() > 0)
}
