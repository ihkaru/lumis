use std::sync::Arc;
use lumis_core::error::{LumisError, Result};
use lumis_core::traits::ChatRepository;
use crate::postgres::PostgresRepository;
use crate::sqlite::SqliteRepository;
use sqlx::{PgPool, SqlitePool};

/// Enterprise Database Factory (EIP Factory & DIP).
/// Dynamically initializes and returns Arc<dyn ChatRepository> based on database URI.
pub struct DatabaseFactory;

impl DatabaseFactory {
    pub async fn create_repository(database_url: &str) -> Result<Arc<dyn ChatRepository>> {
        if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            tracing::info!("Initializing Lumis with PostgreSQL backend...");
            let pool = PgPool::connect(database_url)
                .await
                .map_err(|e| LumisError::Database(format!("Postgres connection failed: {}", e)))?;
            let repo = PostgresRepository::new(pool).await?;
            Ok(Arc::new(repo))
        } else {
            tracing::info!("Initializing Lumis with SQLite WAL backend: {}", database_url);
            let pool = SqlitePool::connect(database_url)
                .await
                .map_err(|e| LumisError::Database(format!("SQLite connection failed: {}", e)))?;
            let repo = SqliteRepository::new(pool).await?;
            Ok(Arc::new(repo))
        }
    }
}
