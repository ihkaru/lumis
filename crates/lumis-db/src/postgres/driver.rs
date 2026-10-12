use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use lumis_core::error::{LumisError, Result};
use lumis_core::traits::{ChatRepository, DriverHealth, RelationalDriver, SqlDialect};
use sqlx::PgPool;
use crate::postgres::{schema, PostgresRepository};

pub struct PostgresDriver {
    pool: PgPool,
    repo: Arc<PostgresRepository>,
}

impl PostgresDriver {
    pub async fn new(pool: PgPool) -> Result<Self> {
        let repo = Arc::new(PostgresRepository::new(pool.clone()).await?);
        Ok(Self { pool, repo })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl RelationalDriver for PostgresDriver {
    fn dialect(&self) -> SqlDialect {
        SqlDialect::Postgres
    }

    async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| LumisError::Database(format!("Postgres ping failed: {}", e)))?;
        Ok(())
    }

    async fn health(&self) -> Result<DriverHealth> {
        let start = Instant::now();
        let ping_res = self.ping().await;
        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(DriverHealth {
            dialect: SqlDialect::Postgres,
            is_alive: ping_res.is_ok(),
            latency_ms,
            active_connections: self.pool.size(),
            max_connections: self.pool.options().get_max_connections(),
        })
    }

    async fn run_migrations(&self) -> Result<()> {
        schema::run_pg_migrations(&self.pool).await
    }

    fn repository(&self) -> Arc<dyn ChatRepository> {
        self.repo.clone()
    }
}
