use lumis_core::error::{LumisError, Result};
use sqlx::PgPool;

pub async fn run_pg_migrations(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id VARCHAR(64) PRIMARY KEY,
            username VARCHAR(128) UNIQUE NOT NULL,
            email VARCHAR(256),
            role SMALLINT NOT NULL DEFAULT 2,
            tenant_id VARCHAR(64),
            preferences JSONB NOT NULL DEFAULT '{}',
            is_active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id VARCHAR(64) PRIMARY KEY,
            user_id VARCHAR(64) NOT NULL REFERENCES users(id),
            title VARCHAR(256) NOT NULL,
            model_id VARCHAR(128) NOT NULL,
            system_prompt TEXT,
            is_pinned BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id VARCHAR(64) PRIMARY KEY,
            session_id VARCHAR(64) NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            role VARCHAR(32) NOT NULL,
            content TEXT NOT NULL,
            model_id VARCHAR(128),
            token_count INTEGER,
            metadata JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_pg_messages_cursor
            ON messages(session_id, created_at DESC, id DESC);

        CREATE TABLE IF NOT EXISTS documents (
            id VARCHAR(64) PRIMARY KEY,
            tenant_id VARCHAR(64) NOT NULL,
            title VARCHAR(256) NOT NULL,
            content_type VARCHAR(64) NOT NULL,
            min_role SMALLINT NOT NULL,
            chunk_count INTEGER NOT NULL,
            created_at TIMESTAMPTZ NOT NULL
        );

        CREATE TABLE IF NOT EXISTS document_chunks (
            id VARCHAR(64) PRIMARY KEY,
            document_id VARCHAR(64) NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            tenant_id VARCHAR(64) NOT NULL,
            chunk_index INTEGER NOT NULL,
            content TEXT NOT NULL,
            min_role SMALLINT NOT NULL,
            embedding_json TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_pg_chunks_role
            ON document_chunks(tenant_id, min_role);
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(())
}
