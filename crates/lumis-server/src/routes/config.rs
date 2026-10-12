use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    pub app_title: String,
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub version: String,
    pub upgrade_channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDoctorReport {
    pub database_healthy: bool,
    pub schema_version: u32,
    pub pending_migrations: usize,
    pub deprecated_configs: Vec<String>,
    pub ready_for_upgrade: bool,
    pub server_version: String,
}

pub async fn get_branding_config() -> impl IntoResponse {
    let app_title = std::env::var("LUMIS_BRAND_TITLE").unwrap_or_else(|_| "Lumis".to_string());
    let logo_url = std::env::var("LUMIS_BRAND_LOGO_URL").ok();
    let primary_color = std::env::var("LUMIS_BRAND_THEME_PRIMARY").ok();
    let upgrade_channel = std::env::var("LUMIS_UPGRADE_CHANNEL").unwrap_or_else(|_| "stable".to_string());

    let config = BrandingConfig {
        app_title,
        logo_url,
        primary_color,
        version: env!("CARGO_PKG_VERSION").to_string(),
        upgrade_channel,
    };

    Json(config)
}

pub async fn get_system_doctor(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = state.driver.health().await.map(|h| h.is_alive).unwrap_or(false);

    // Validate if any deprecated environment variables are in use
    let mut deprecated = Vec::new();
    if std::env::var("OLD_AUTH_SECRET").is_ok() {
        deprecated.push("OLD_AUTH_SECRET (use LUMIS_JWT_SECRET)".to_string());
    }
    if std::env::var("MONGODB_URI").is_ok() {
        deprecated.push("MONGODB_URI (Lumis uses RelationalDriver PostgreSQL/SQLite)".to_string());
    }

    let report = SystemDoctorReport {
        database_healthy: db_ok,
        schema_version: 1,
        pending_migrations: 0,
        deprecated_configs: deprecated.clone(),
        ready_for_upgrade: db_ok && deprecated.is_empty(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(report)
}
