pub mod auth;
pub mod chat;
pub mod health;
pub mod metrics;
pub mod stream;

use axum::routing::get;
use axum::Router;

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health & Diagnostics
        .route("/health", get(health::health_check))
        .route("/health/db", get(health::db_health))
        // Prometheus Metrics & FinOps Telemetry
        .route("/metrics", get(metrics::get_metrics))
        .route("/api/finops/summary", get(metrics::get_finops_summary))
        // Auth, Identity & Governance
        .route("/api/auth/me", get(auth::auth_me))
        .route("/api/admin/roles", get(auth::list_roles))
        // Chat & Keyset Pagination
        .route(
            "/api/sessions",
            get(chat::list_sessions).post(chat::create_session),
        )
        .route(
            "/api/sessions/{session_id}/messages",
            get(chat::list_messages).post(chat::send_message),
        )

        // Zero-Copy SSE Stream
        .route("/api/stream", get(stream::sse_handler))
        .with_state(state)
}

