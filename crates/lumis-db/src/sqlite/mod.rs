pub mod driver;
pub mod identity;
pub mod mcp;
pub mod mcp_oauth;
pub mod mcp_quota;
pub mod mcp_tools;
pub mod memory;
pub mod message;
pub mod schema;
pub mod scim;
pub mod session;
pub mod user;

pub use driver::SqliteDriver;


use async_trait::async_trait;
use lumis_core::error::Result;
use lumis_core::models::{
    CursorPage, CursorQuery, Message, NewMessage, NewSession, NewUserMemory, Session, User, UserMemory,
};
use lumis_core::traits::ChatRepository;
use sqlx::SqlitePool;

pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn new(pool: SqlitePool) -> Result<Self> {
        schema::run_sqlite_migrations(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl ChatRepository for SqliteRepository {
    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        user::get_user_by_id(&self.pool, id).await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| lumis_core::error::LumisError::Database(e.to_string()))?;

        if let Some((id,)) = row {
            user::get_user_by_id(&self.pool, &id).await
        } else {
            Ok(None)
        }
    }

    async fn upsert_user(&self, u: &User) -> Result<User> {
        user::upsert_user(&self.pool, u).await
    }

    async fn list_users(&self, limit: usize) -> Result<Vec<User>> {
        user::list_users(&self.pool, limit).await
    }

    async fn create_session(&self, new_session: &NewSession) -> Result<Session> {
        session::create_session(&self.pool, new_session).await
    }

    async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        session::get_session(&self.pool, id).await
    }

    async fn update_session_title(&self, session_id: &str, title: &str) -> Result<bool> {
        session::update_session_title(&self.pool, session_id, title).await
    }

    async fn update_session_model(&self, session_id: &str, model_id: &str) -> Result<bool> {
        session::update_session_model(&self.pool, session_id, model_id).await
    }

    async fn list_sessions(
        &self,
        user_id: &str,
        query: &CursorQuery,
    ) -> Result<CursorPage<Session>> {
        session::list_sessions(&self.pool, user_id, query).await
    }

    async fn delete_session(&self, id: &str) -> Result<bool> {
        session::delete_session(&self.pool, id).await
    }

    async fn append_message(&self, new_message: &NewMessage) -> Result<Message> {
        message::append_message(&self.pool, new_message).await
    }

    async fn list_messages(
        &self,
        session_id: &str,
        query: &CursorQuery,
    ) -> Result<CursorPage<Message>> {
        message::list_messages(&self.pool, session_id, query).await
    }

    async fn search_messages(
        &self,
        user_id: &str,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<Message>> {
        message::search_messages(&self.pool, user_id, query_text, limit).await
    }

    async fn list_user_memories(&self, user_id: &str) -> Result<Vec<UserMemory>> {
        memory::list_user_memories(&self.pool, user_id).await
    }

    async fn add_user_memory(&self, new_mem: &NewUserMemory) -> Result<UserMemory> {
        memory::add_user_memory(&self.pool, new_mem).await
    }

    async fn delete_user_memory(&self, id: &str, user_id: &str) -> Result<bool> {
        memory::delete_user_memory(&self.pool, id, user_id).await
    }

    async fn find_identity(&self, provider: &str, provider_subject_id: &str) -> Result<Option<lumis_core::models::UserIdentity>> {
        identity::find_identity(&self.pool, provider, provider_subject_id).await
    }

    async fn link_identity(&self, idty: &lumis_core::models::UserIdentity) -> Result<lumis_core::models::UserIdentity> {
        identity::link_identity(&self.pool, idty).await
    }

    async fn list_identities_by_user(&self, user_id: &str) -> Result<Vec<lumis_core::models::UserIdentity>> {
        identity::list_identities_by_user(&self.pool, user_id).await
    }

    async fn unlink_identity(&self, user_id: &str, provider: &str) -> Result<bool> {
        identity::unlink_identity(&self.pool, user_id, provider).await
    }

    async fn list_scim_users(&self) -> Result<Vec<lumis_core::models::scim::ScimUser>> {
        scim::list_scim_users(&self.pool).await
    }

    async fn get_scim_user(&self, id: &str) -> Result<Option<lumis_core::models::scim::ScimUser>> {
        scim::get_scim_user(&self.pool, id).await
    }

    async fn upsert_scim_user(&self, user: &lumis_core::models::scim::ScimUser) -> Result<lumis_core::models::scim::ScimUser> {
        scim::upsert_scim_user(&self.pool, user).await
    }

    async fn update_scim_user_status(&self, id: &str, active: bool) -> Result<bool> {
        scim::update_scim_user_status(&self.pool, id, active).await
    }

    async fn delete_scim_user(&self, id: &str) -> Result<bool> {
        scim::delete_scim_user(&self.pool, id).await
    }

    async fn list_scim_groups(&self) -> Result<Vec<lumis_core::models::scim::ScimGroup>> {
        scim::list_scim_groups(&self.pool).await
    }

    async fn get_scim_group(&self, id: &str) -> Result<Option<lumis_core::models::scim::ScimGroup>> {
        scim::get_scim_group(&self.pool, id).await
    }

    async fn upsert_scim_group(&self, group: &lumis_core::models::scim::ScimGroup) -> Result<lumis_core::models::scim::ScimGroup> {
        scim::upsert_scim_group(&self.pool, group).await
    }

    async fn delete_scim_group(&self, id: &str) -> Result<bool> {
        scim::delete_scim_group(&self.pool, id).await
    }

    // Model Context Protocol (MCP 2026) Server & Tool Persistence
    async fn create_mcp_server(&self, server: &lumis_core::models::mcp::McpServerRecord) -> Result<lumis_core::models::mcp::McpServerRecord> {
        mcp::create_mcp_server(&self.pool, server).await
    }

    async fn get_mcp_server(&self, id: &str, tenant_id: &str) -> Result<Option<lumis_core::models::mcp::McpServerRecord>> {
        mcp::get_mcp_server(&self.pool, id, tenant_id).await
    }

    async fn list_mcp_servers(&self, tenant_id: &str) -> Result<Vec<lumis_core::models::mcp::McpServerRecord>> {
        mcp::list_mcp_servers(&self.pool, tenant_id).await
    }

    async fn list_mcp_servers_for_user(&self, tenant_id: &str, user_id: &str, user_role: &str) -> Result<Vec<lumis_core::models::mcp::McpServerRecord>> {
        mcp::list_mcp_servers_for_user(&self.pool, tenant_id, user_id, user_role).await
    }

    async fn update_mcp_server(&self, server: &lumis_core::models::mcp::McpServerRecord) -> Result<lumis_core::models::mcp::McpServerRecord> {
        mcp::update_mcp_server(&self.pool, server).await
    }

    async fn delete_mcp_server(&self, id: &str, tenant_id: &str) -> Result<bool> {
        mcp::delete_mcp_server(&self.pool, id, tenant_id).await
    }

    async fn sync_mcp_tools(&self, server_id: &str, tools: &[lumis_core::models::mcp::McpToolRecord]) -> Result<()> {
        mcp::sync_mcp_tools(&self.pool, server_id, tools).await
    }

    async fn list_mcp_tools(&self, server_id: &str) -> Result<Vec<lumis_core::models::mcp::McpToolRecord>> {
        mcp::list_mcp_tools(&self.pool, server_id).await
    }

    async fn log_mcp_invocation(&self, invocation: &lumis_core::models::mcp::McpToolInvocationRecord) -> Result<()> {
        mcp::log_mcp_invocation(&self.pool, invocation).await
    }

    // MCP 2.1 & OAuth 2.1 User Tokens and Preferences
    async fn save_user_mcp_oauth_token(&self, token: &lumis_core::models::mcp::UserMcpOAuthTokenRecord) -> Result<lumis_core::models::mcp::UserMcpOAuthTokenRecord> {
        mcp_oauth::save_user_mcp_oauth_token(&self.pool, token).await
    }

    async fn get_user_mcp_oauth_token(&self, user_id: &str, server_id: &str) -> Result<Option<lumis_core::models::mcp::UserMcpOAuthTokenRecord>> {
        mcp_oauth::get_user_mcp_oauth_token(&self.pool, user_id, server_id).await
    }

    async fn delete_user_mcp_oauth_token(&self, user_id: &str, server_id: &str) -> Result<bool> {
        mcp_oauth::delete_user_mcp_oauth_token(&self.pool, user_id, server_id).await
    }

    async fn revoke_tokens_by_session_or_user(&self, user_id: &str, idp_session_id: Option<&str>) -> Result<u64> {
        mcp_oauth::revoke_tokens_by_session_or_user(&self.pool, user_id, idp_session_id).await
    }

    async fn get_user_mcp_preference(&self, user_id: &str, server_id: &str) -> Result<Option<bool>> {
        mcp_oauth::get_user_mcp_preference(&self.pool, user_id, server_id).await
    }

    async fn set_user_mcp_preference(&self, user_id: &str, server_id: &str, is_enabled: bool) -> Result<()> {
        mcp_oauth::set_user_mcp_preference(&self.pool, user_id, server_id, is_enabled).await
    }

    async fn list_user_mcp_preferences(&self, user_id: &str) -> Result<Vec<lumis_core::models::mcp::UserMcpPreferenceRecord>> {
        mcp_oauth::list_user_mcp_preferences(&self.pool, user_id).await
    }

    async fn check_and_increment_mcp_quota(&self, user_id: &str, server_id: &str, daily_limit: i32) -> Result<bool> {
        mcp_quota::check_and_increment_mcp_quota(&self.pool, user_id, server_id, daily_limit).await
    }

    async fn get_mcp_quota_usage(&self, user_id: &str, server_id: &str) -> Result<i32> {
        mcp_quota::get_mcp_quota_usage(&self.pool, user_id, server_id).await
    }
}
