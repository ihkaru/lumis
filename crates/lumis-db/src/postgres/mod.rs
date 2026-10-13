pub mod driver;
pub mod identity;
pub mod memory;
pub mod message;
pub mod schema;
pub mod scim;
pub mod session;
pub mod user;

pub use driver::PostgresDriver;


use async_trait::async_trait;
use lumis_core::error::Result;
use lumis_core::models::{
    CursorPage, CursorQuery, Message, NewMessage, NewSession, NewUserMemory, Session, User, UserMemory,
};
use lumis_core::traits::ChatRepository;
use sqlx::PgPool;

pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub async fn new(pool: PgPool) -> Result<Self> {
        schema::run_pg_migrations(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl ChatRepository for PostgresRepository {
    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        user::get_user_by_id(&self.pool, id).await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE username = $1")
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
}
