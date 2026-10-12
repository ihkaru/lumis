pub mod audit;
pub mod auth;
pub mod chat;
pub mod health;
pub mod knowledge;
pub mod mcp;
pub mod metrics;
pub mod stream;

use axum::routing::get;
use axum::Router;

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let router = Router::new()
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
        // Enterprise Knowledge Vault & Role-Gated Hybrid RAG
        .route(
            "/api/knowledge/documents",
            get(knowledge::list_documents).post(knowledge::ingest_document),
        )
        .route(
            "/api/knowledge/documents/{document_id}",
            axum::routing::delete(knowledge::delete_document),
        )
        .route("/api/knowledge/query", axum::routing::post(knowledge::query_knowledge))

        // Model Context Protocol (MCP 2026) & Asynchronous Tasks Extension
        .route("/api/mcp/tools", get(mcp::list_tools))
        .route("/api/mcp/tools/call", axum::routing::post(mcp::call_tool))
        .route("/api/mcp/tasks", axum::routing::post(mcp::spawn_task))
        .route("/api/mcp/tasks/{task_id}", get(mcp::get_task_status))
        .route("/api/mcp/tasks/{task_id}/cancel", axum::routing::post(mcp::cancel_task))

        // Centralized SIEM Auditing & Emergency SOC Killswitch
        .route("/api/admin/audit/events", get(audit::get_audit_events))
        .route(
            "/api/admin/emergency/killswitch",
            get(audit::get_killswitch).post(audit::set_killswitch),
        )

        // Zero-Copy SSE Stream
        .route("/api/stream", get(stream::sse_handler))
        .with_state(state);

    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "ui/dist".to_string());
    if std::path::Path::new(&static_dir).exists() {
        router.fallback_service(tower_http::services::ServeDir::new(static_dir))
    } else {
        router
    }
}

