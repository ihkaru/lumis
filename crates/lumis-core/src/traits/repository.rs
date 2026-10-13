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
}

