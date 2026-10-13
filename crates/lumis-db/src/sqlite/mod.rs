pub mod driver;
pub mod memory;
pub mod message;
pub mod schema;
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

    async fn create_session(&self, new_session: &NewSession) -> Result<Session> {
        session::create_session(&self.pool, new_session).await
    }

    async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        session::get_session(&self.pool, id).await
    }

    async fn update_session_title(&self, session_id: &str, title: &str) -> Result<bool> {
        session::update_session_title(&self.pool, session_id, title).await
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
}
