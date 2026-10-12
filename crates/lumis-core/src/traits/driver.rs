use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::traits::repository::ChatRepository;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqlDialect {
    Sqlite,
    Postgres,
}

impl SqlDialect {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
        }
    }

    pub fn placeholder(&self, index: usize) -> String {
        match self {
            Self::Sqlite => "?".to_string(),
            Self::Postgres => format!("${}", index),
        }
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self, Self::Postgres)
    }

    pub fn is_sqlite(&self) -> bool {
        matches!(self, Self::Sqlite)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverHealth {
    pub dialect: SqlDialect,
    pub is_alive: bool,
    pub latency_ms: u64,
    pub active_connections: u32,
    pub max_connections: u32,
}

/// Interface-First Relational Driver Pattern (EIP Channel Adapter).
/// Decouples persistence from specific relational database engines (Postgres & SQLite).
#[async_trait]
pub trait RelationalDriver: Send + Sync {
    /// Returns the active SQL dialect.
    fn dialect(&self) -> SqlDialect;

    /// Verifies live connectivity with the underlying database.
    async fn ping(&self) -> Result<()>;

    /// Returns detailed driver health and latency metrics.
    async fn health(&self) -> Result<DriverHealth>;

    /// Executes self-bootstrapping schema migrations.
    async fn run_migrations(&self) -> Result<()>;

    /// Returns the associated ChatRepository implementation.
    fn repository(&self) -> Arc<dyn ChatRepository>;
}
