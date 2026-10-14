use async_trait::async_trait;
use crate::error::Result;
use crate::models::{
    CursorPage, CursorQuery, Message, NewMessage, NewSession, Session, User,
};

/// Interface-First Pluggable Chat Repository.
/// Supports SQLite, PostgreSQL, and Valkey backends interchangeably (SOLID DIP & LSP).
#[async_trait]
pub trait ChatRepository: Send + Sync {
    // User Operations
    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn upsert_user(&self, user: &User) -> Result<User>;
    async fn list_users(&self, limit: usize) -> Result<Vec<User>>;

    // Session Operations
    async fn create_session(&self, new_session: &NewSession) -> Result<Session>;
    async fn get_session(&self, id: &str) -> Result<Option<Session>>;
    async fn update_session_title(&self, session_id: &str, title: &str) -> Result<bool>;
    async fn update_session_model(&self, session_id: &str, model_id: &str) -> Result<bool>;
    async fn list_sessions(
        &self,
        user_id: &str,
        query: &CursorQuery,
    ) -> Result<CursorPage<Session>>;
    async fn delete_session(&self, id: &str) -> Result<bool>;

    // Message Operations (Keyset pagination O(log N))
    async fn append_message(&self, new_message: &NewMessage) -> Result<Message>;
    async fn list_messages(
        &self,
        session_id: &str,
        query: &CursorQuery,
    ) -> Result<CursorPage<Message>>;

    // Search Operations (Full-Text Inverted Index)
    async fn search_messages(
        &self,
        user_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<Message>>;

    // Federated Identity Operations (Phase 2 Multi-IdP Decoupled Architecture)
    async fn find_identity(&self, provider: &str, provider_subject_id: &str) -> Result<Option<crate::models::UserIdentity>>;
    async fn link_identity(&self, identity: &crate::models::UserIdentity) -> Result<crate::models::UserIdentity>;
    async fn list_identities_by_user(&self, user_id: &str) -> Result<Vec<crate::models::UserIdentity>>;
    async fn unlink_identity(&self, user_id: &str, provider: &str) -> Result<bool>;

    // User Memory Operations (Enterprise Long-Term Memory)
    async fn list_user_memories(&self, user_id: &str) -> Result<Vec<crate::models::UserMemory>>;
    async fn add_user_memory(&self, new_mem: &crate::models::NewUserMemory) -> Result<crate::models::UserMemory>;
    async fn delete_user_memory(&self, id: &str, user_id: &str) -> Result<bool>;

    // SCIM 2.0 Resource Persistence (RFC 7643/7644)
    async fn list_scim_users(&self) -> Result<Vec<crate::models::scim::ScimUser>>;
    async fn get_scim_user(&self, id: &str) -> Result<Option<crate::models::scim::ScimUser>>;
    async fn upsert_scim_user(&self, user: &crate::models::scim::ScimUser) -> Result<crate::models::scim::ScimUser>;
    async fn update_scim_user_status(&self, id: &str, active: bool) -> Result<bool>;
    async fn delete_scim_user(&self, id: &str) -> Result<bool>;

    async fn list_scim_groups(&self) -> Result<Vec<crate::models::scim::ScimGroup>>;
    async fn get_scim_group(&self, id: &str) -> Result<Option<crate::models::scim::ScimGroup>>;
    async fn upsert_scim_group(&self, group: &crate::models::scim::ScimGroup) -> Result<crate::models::scim::ScimGroup>;
    async fn delete_scim_group(&self, id: &str) -> Result<bool>;

    // Model Context Protocol (MCP 2026) Server & Tool Persistence
    async fn create_mcp_server(&self, server: &crate::models::mcp::McpServerRecord) -> Result<crate::models::mcp::McpServerRecord>;
    async fn get_mcp_server(&self, id: &str, tenant_id: &str) -> Result<Option<crate::models::mcp::McpServerRecord>>;
    async fn list_mcp_servers(&self, tenant_id: &str) -> Result<Vec<crate::models::mcp::McpServerRecord>>;
    async fn list_mcp_servers_for_user(&self, tenant_id: &str, user_id: &str, user_role: &str) -> Result<Vec<crate::models::mcp::McpServerRecord>>;
    async fn update_mcp_server(&self, server: &crate::models::mcp::McpServerRecord) -> Result<crate::models::mcp::McpServerRecord>;
    async fn delete_mcp_server(&self, id: &str, tenant_id: &str) -> Result<bool>;
    async fn sync_mcp_tools(&self, server_id: &str, tools: &[crate::models::mcp::McpToolRecord]) -> Result<()>;
    async fn list_mcp_tools(&self, server_id: &str) -> Result<Vec<crate::models::mcp::McpToolRecord>>;
    async fn log_mcp_invocation(&self, invocation: &crate::models::mcp::McpToolInvocationRecord) -> Result<()>;

    // MCP 2.1 & OAuth 2.1 User Tokens and Preferences
    async fn save_user_mcp_oauth_token(&self, token: &crate::models::mcp::UserMcpOAuthTokenRecord) -> Result<crate::models::mcp::UserMcpOAuthTokenRecord>;
    async fn get_user_mcp_oauth_token(&self, user_id: &str, server_id: &str) -> Result<Option<crate::models::mcp::UserMcpOAuthTokenRecord>>;
    async fn delete_user_mcp_oauth_token(&self, user_id: &str, server_id: &str) -> Result<bool>;
    async fn revoke_tokens_by_session_or_user(&self, user_id: &str, idp_session_id: Option<&str>) -> Result<u64>;
    async fn get_user_mcp_preference(&self, user_id: &str, server_id: &str) -> Result<Option<bool>>;
    async fn set_user_mcp_preference(&self, user_id: &str, server_id: &str, is_enabled: bool) -> Result<()>;
    async fn list_user_mcp_preferences(&self, user_id: &str) -> Result<Vec<crate::models::mcp::UserMcpPreferenceRecord>>;

    // MCP Quota and Rate Limiting
    async fn check_and_increment_mcp_quota(&self, user_id: &str, server_id: &str, daily_limit: i32) -> Result<bool>;
    async fn get_mcp_quota_usage(&self, user_id: &str, server_id: &str) -> Result<i32>;

    // Dynamic Task Planning & Goal State Machine Operations (Coding Agent Pattern)
    async fn create_session_task(&self, task: &crate::models::CreateTaskInput) -> Result<crate::models::SessionTask>;
    async fn get_session_task(&self, task_id: &str) -> Result<Option<crate::models::SessionTask>>;
    async fn update_session_task(&self, task_id: &str, update: &crate::models::UpdateTaskInput) -> Result<Option<crate::models::SessionTask>>;
    async fn list_session_tasks(&self, session_id: &str) -> Result<Vec<crate::models::SessionTask>>;
    async fn delete_session_task(&self, task_id: &str) -> Result<bool>;
    async fn clear_session_tasks(&self, session_id: &str) -> Result<u64>;
}

