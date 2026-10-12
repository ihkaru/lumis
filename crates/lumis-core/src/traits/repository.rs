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

    // Session Operations
    async fn create_session(&self, new_session: &NewSession) -> Result<Session>;
    async fn get_session(&self, id: &str) -> Result<Option<Session>>;
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
}
