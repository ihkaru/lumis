#![deny(unsafe_code)]

//! # Lumis Database Layer
//! Pluggable database implementations (SQLite & PostgreSQL) implementing ChatRepository trait.
//! Interface-First, Dependency Inversion Principle (DIP).

pub mod factory;
pub mod postgres;
pub mod sqlite;

pub use factory::DatabaseFactory;
pub use postgres::{PostgresDriver, PostgresRepository};
pub use sqlite::{SqliteDriver, SqliteRepository};

