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
            password_hash VARCHAR(256),
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

        CREATE TABLE IF NOT EXISTS user_memories (
            id VARCHAR(64) PRIMARY KEY,
            user_id VARCHAR(64) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            category VARCHAR(64) NOT NULL DEFAULT 'preference',
            content TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_pg_user_memories_user
            ON user_memories(user_id, created_at DESC);

        CREATE TABLE IF NOT EXISTS user_identities (
            id VARCHAR(64) PRIMARY KEY,
            user_id VARCHAR(64) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            provider VARCHAR(64) NOT NULL,
            provider_subject_id VARCHAR(256) NOT NULL,
            credential_data JSONB,
            last_login_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            CONSTRAINT uq_pg_user_identity UNIQUE(provider, provider_subject_id)
        );

        CREATE INDEX IF NOT EXISTS idx_pg_user_identities_user_id
            ON user_identities(user_id);

        CREATE TABLE IF NOT EXISTS scim_users (
            id VARCHAR(64) PRIMARY KEY,
            external_id VARCHAR(256),
            username VARCHAR(256) UNIQUE NOT NULL,
            display_name VARCHAR(256),
            active BOOLEAN NOT NULL DEFAULT TRUE,
            emails JSONB NOT NULL DEFAULT '[]',
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        );

        CREATE TABLE IF NOT EXISTS scim_groups (
            id VARCHAR(64) PRIMARY KEY,
            display_name VARCHAR(256) UNIQUE NOT NULL,
            members JSONB NOT NULL DEFAULT '[]',
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        );

        CREATE TABLE IF NOT EXISTS mcp_servers (
            id VARCHAR(64) PRIMARY KEY,
            tenant_id VARCHAR(64) NOT NULL,
            created_by VARCHAR(64) NOT NULL REFERENCES users(id),
            name VARCHAR(64) NOT NULL,
            display_title VARCHAR(128) NOT NULL,
            description TEXT,
            transport_type VARCHAR(24) NOT NULL,
            endpoint_url TEXT,
            command_bin VARCHAR(128),
            command_args TEXT,
            auth_type VARCHAR(24) NOT NULL,
            encrypted_credentials TEXT,
            is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            risk_tier SMALLINT NOT NULL DEFAULT 1,
            scope_type VARCHAR(24) NOT NULL DEFAULT 'system',
            owner_user_id VARCHAR(64) REFERENCES users(id),
            is_mandatory BOOLEAN NOT NULL DEFAULT FALSE,
            allowed_roles TEXT NOT NULL DEFAULT '["admin","user","auditor"]',
            oauth_config TEXT,
            ssrf_policy VARCHAR(32) NOT NULL DEFAULT 'strict',
            egress_allowlist TEXT,
            sandbox_type VARCHAR(32) NOT NULL DEFAULT 'restricted_process',
            max_memory_mb INTEGER NOT NULL DEFAULT 256,
            timeout_seconds INTEGER NOT NULL DEFAULT 30,
            dlp_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            dlp_rules TEXT NOT NULL DEFAULT '["credit_card","nik","phone","email","jwt"]',
            rate_limit_rpm INTEGER NOT NULL DEFAULT 60,
            daily_request_limit INTEGER NOT NULL DEFAULT 1000,
            kms_provider VARCHAR(32) NOT NULL DEFAULT 'local',
            kms_key_id TEXT,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            CONSTRAINT uq_pg_mcp_server_tenant_name UNIQUE (tenant_id, name)
        );

        CREATE TABLE IF NOT EXISTS mcp_tools (
            id VARCHAR(64) PRIMARY KEY,
            server_id VARCHAR(64) NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
            name VARCHAR(64) NOT NULL,
            description TEXT,
            input_schema_json TEXT NOT NULL DEFAULT '{}',
            risk_tier SMALLINT NOT NULL DEFAULT 1,
            requires_approval BOOLEAN NOT NULL DEFAULT FALSE,
            updated_at TIMESTAMPTZ NOT NULL,
            CONSTRAINT uq_pg_mcp_tool_server_name UNIQUE (server_id, name)
        );

        CREATE TABLE IF NOT EXISTS mcp_tool_invocations (
            id VARCHAR(64) PRIMARY KEY,
            tenant_id VARCHAR(64) NOT NULL,
            user_id VARCHAR(64) NOT NULL REFERENCES users(id),
            server_id VARCHAR(64) NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
            tool_name VARCHAR(64) NOT NULL,
            status VARCHAR(24) NOT NULL,
            execution_time_ms INTEGER NOT NULL,
            approval_token VARCHAR(64),
            payload_redacted BOOLEAN NOT NULL DEFAULT FALSE,
            redaction_summary TEXT,
            created_at TIMESTAMPTZ NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_pg_mcp_invocations_tenant_user
            ON mcp_tool_invocations (tenant_id, user_id, created_at DESC);

        CREATE TABLE IF NOT EXISTS user_mcp_oauth_tokens (
            id VARCHAR(64) PRIMARY KEY,
            user_id VARCHAR(64) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            server_id VARCHAR(64) NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
            access_token_encrypted TEXT NOT NULL,
            refresh_token_encrypted TEXT,
            token_type VARCHAR(24) NOT NULL DEFAULT 'Bearer',
            scope TEXT,
            expires_at BIGINT,
            is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
            revoked_at TIMESTAMPTZ,
            idp_session_id VARCHAR(128),
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            CONSTRAINT uq_pg_user_mcp_oauth UNIQUE (user_id, server_id)
        );

        CREATE TABLE IF NOT EXISTS user_mcp_preferences (
            user_id VARCHAR(64) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            server_id VARCHAR(64) NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
            is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            updated_at TIMESTAMPTZ NOT NULL,
            PRIMARY KEY (user_id, server_id)
        );

        CREATE TABLE IF NOT EXISTS mcp_usage_quotas (
            id VARCHAR(64) PRIMARY KEY,
            user_id VARCHAR(64) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            server_id VARCHAR(64) NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
            window_date VARCHAR(24) NOT NULL,
            request_count INTEGER NOT NULL DEFAULT 0,
            updated_at TIMESTAMPTZ NOT NULL,
            CONSTRAINT uq_pg_mcp_quota UNIQUE (user_id, server_id, window_date)
        );

        CREATE TABLE IF NOT EXISTS session_tasks (
            id VARCHAR(64) PRIMARY KEY,
            session_id VARCHAR(64) NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            parent_task_id VARCHAR(64) REFERENCES session_tasks(id) ON DELETE CASCADE,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            status VARCHAR(32) NOT NULL DEFAULT 'pending',
            order_index INTEGER NOT NULL DEFAULT 0,
            result_summary TEXT,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_pg_session_tasks_session ON session_tasks(session_id, order_index);
        CREATE INDEX IF NOT EXISTS idx_pg_session_tasks_status ON session_tasks(session_id, status);
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    // Idempotent migration for existing PostgreSQL databases
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash VARCHAR(256);").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS scope_type VARCHAR(24) NOT NULL DEFAULT 'system';").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS owner_user_id VARCHAR(64) REFERENCES users(id);").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS is_mandatory BOOLEAN NOT NULL DEFAULT FALSE;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS allowed_roles TEXT NOT NULL DEFAULT '[\"admin\",\"user\",\"auditor\"]';").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS oauth_config TEXT;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS ssrf_policy VARCHAR(32) NOT NULL DEFAULT 'strict';").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS egress_allowlist TEXT;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS sandbox_type VARCHAR(32) NOT NULL DEFAULT 'restricted_process';").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS max_memory_mb INTEGER NOT NULL DEFAULT 256;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS timeout_seconds INTEGER NOT NULL DEFAULT 30;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS dlp_enabled BOOLEAN NOT NULL DEFAULT TRUE;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS dlp_rules TEXT NOT NULL DEFAULT '[\"credit_card\",\"nik\",\"phone\",\"email\",\"jwt\"]';").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS rate_limit_rpm INTEGER NOT NULL DEFAULT 60;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS daily_request_limit INTEGER NOT NULL DEFAULT 1000;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS kms_provider VARCHAR(32) NOT NULL DEFAULT 'local';").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_servers ADD COLUMN IF NOT EXISTS kms_key_id TEXT;").execute(pool).await;

    let _ = sqlx::query("ALTER TABLE mcp_tool_invocations ADD COLUMN IF NOT EXISTS payload_redacted BOOLEAN NOT NULL DEFAULT FALSE;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE mcp_tool_invocations ADD COLUMN IF NOT EXISTS redaction_summary TEXT;").execute(pool).await;

    let _ = sqlx::query("ALTER TABLE user_mcp_oauth_tokens ADD COLUMN IF NOT EXISTS is_revoked BOOLEAN NOT NULL DEFAULT FALSE;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE user_mcp_oauth_tokens ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMPTZ;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE user_mcp_oauth_tokens ADD COLUMN IF NOT EXISTS idp_session_id VARCHAR(128);").execute(pool).await;

    // Idempotent backfill from legacy users.password_hash into user_identities
    let _ = sqlx::query(
        r#"
        INSERT INTO user_identities (id, user_id, provider, provider_subject_id, credential_data, last_login_at, created_at, updated_at)
        SELECT id, id, 'local', username, jsonb_build_object('password_hash', password_hash), updated_at, created_at, updated_at
        FROM users
        WHERE password_hash IS NOT NULL
        ON CONFLICT (provider, provider_subject_id) DO NOTHING;
        "#
    ).execute(pool).await;

    Ok(())
}
