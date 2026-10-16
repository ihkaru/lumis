use std::sync::Arc;
use lumis_core::error::{LumisError, Result};
use lumis_core::traits::{ChatRepository, RelationalDriver};
use crate::postgres::PostgresDriver;
use crate::sqlite::SqliteDriver;
use sqlx::PgPool;

/// Enterprise Database Factory (EIP Factory & DIP).
/// Dynamically initializes and returns Arc<dyn RelationalDriver> or Arc<dyn ChatRepository>
/// based on database URI scheme.
pub struct DatabaseFactory;

impl DatabaseFactory {
    pub async fn create_driver(database_url: &str) -> Result<Arc<dyn RelationalDriver>> {
        if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            tracing::info!("Initializing Lumis with PostgreSQL driver...");
            let pool = PgPool::connect(database_url)
                .await
                .map_err(|e| LumisError::Database(format!("Postgres connection failed: {}", e)))?;
            let driver = PostgresDriver::new(pool).await?;
            Ok(Arc::new(driver))
        } else {
            tracing::info!("Initializing Lumis with SQLite WAL driver: {}", database_url);
            let options = database_url
                .parse::<sqlx::sqlite::SqliteConnectOptions>()
                .map_err(|e| LumisError::Database(format!("Invalid SQLite URL: {}", e)))?
                .create_if_missing(true);
            let max_conn = if database_url.contains(":memory:") { 1 } else { 10 };
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(max_conn)
                .connect_with(options)
                .await
                .map_err(|e| LumisError::Database(format!("SQLite connection failed: {}", e)))?;
            let driver = SqliteDriver::new(pool).await?;
            Ok(Arc::new(driver))
        }
    }

    pub async fn create_repository(database_url: &str) -> Result<Arc<dyn ChatRepository>> {
        let driver = Self::create_driver(database_url).await?;
        Ok(driver.repository())
    }
}

