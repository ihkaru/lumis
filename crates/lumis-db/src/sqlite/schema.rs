use lumis_core::error::{LumisError, Result};
use sqlx::SqlitePool;

pub async fn run_sqlite_migrations(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;

        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT,
            role INTEGER NOT NULL DEFAULT 2,
            tenant_id TEXT,
            preferences TEXT NOT NULL DEFAULT '{}',
            is_active INTEGER NOT NULL DEFAULT 1,
            password_hash TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            title TEXT NOT NULL,
            model_id TEXT NOT NULL,
            system_prompt TEXT,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            model_id TEXT,
            token_count INTEGER,
            metadata TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_messages_cursor
            ON messages(session_id, created_at DESC, id DESC);

        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            title TEXT NOT NULL,
            content_type TEXT NOT NULL,
            min_role INTEGER NOT NULL,
            chunk_count INTEGER NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS document_chunks (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            tenant_id TEXT NOT NULL,
            chunk_index INTEGER NOT NULL,
            content TEXT NOT NULL,
            min_role INTEGER NOT NULL,
            embedding_json TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_chunks_role
            ON document_chunks(tenant_id, min_role);

        CREATE TABLE IF NOT EXISTS user_memories (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            category TEXT NOT NULL DEFAULT 'preference',
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_user_memories_user
            ON user_memories(user_id, created_at DESC);

        CREATE TABLE IF NOT EXISTS user_identities (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            provider TEXT NOT NULL,
            provider_subject_id TEXT NOT NULL,
            credential_data TEXT,
            last_login_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT uq_user_identity UNIQUE(provider, provider_subject_id)
        );

        CREATE INDEX IF NOT EXISTS idx_user_identities_user_id
            ON user_identities(user_id);

        CREATE TABLE IF NOT EXISTS scim_users (
            id TEXT PRIMARY KEY,
            external_id TEXT,
            username TEXT UNIQUE NOT NULL,
            display_name TEXT,
            active INTEGER NOT NULL DEFAULT 1,
            emails TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS scim_groups (
            id TEXT PRIMARY KEY,
            display_name TEXT UNIQUE NOT NULL,
            members TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS mcp_servers (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            created_by TEXT NOT NULL REFERENCES users(id),
            name TEXT NOT NULL,
            display_title TEXT NOT NULL,
            description TEXT,
            transport_type TEXT NOT NULL,
            endpoint_url TEXT,
            command_bin TEXT,
            command_args TEXT,
            auth_type TEXT NOT NULL,
            encrypted_credentials TEXT,
            is_enabled INTEGER NOT NULL DEFAULT 1,
            risk_tier INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT uq_mcp_server_tenant_name UNIQUE (tenant_id, name)
        );

        CREATE TABLE IF NOT EXISTS mcp_tools (
            id TEXT PRIMARY KEY,
            server_id TEXT NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            description TEXT,
            input_schema_json TEXT NOT NULL DEFAULT '{}',
            risk_tier INTEGER NOT NULL DEFAULT 1,
            requires_approval INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL,
            CONSTRAINT uq_mcp_tool_server_name UNIQUE (server_id, name)
        );

        CREATE TABLE IF NOT EXISTS mcp_tool_invocations (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            user_id TEXT NOT NULL REFERENCES users(id),
            server_id TEXT NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
            tool_name TEXT NOT NULL,
            status TEXT NOT NULL,
            execution_time_ms INTEGER NOT NULL,
            approval_token TEXT,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_mcp_invocations_tenant_user
            ON mcp_tool_invocations (tenant_id, user_id, created_at DESC);
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    // Idempotent migration for existing SQLite databases
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN password_hash TEXT;").execute(pool).await;

    // Idempotent backfill from legacy users.password_hash into user_identities
    let _ = sqlx::query(
        r#"
        INSERT OR IGNORE INTO user_identities (id, user_id, provider, provider_subject_id, credential_data, last_login_at, created_at, updated_at)
        SELECT id, id, 'local', username, json_object('password_hash', password_hash), updated_at, created_at, updated_at
        FROM users
        WHERE password_hash IS NOT NULL;
        "#
    ).execute(pool).await;

    Ok(())
}
