pub mod admin_users;
pub mod approval;
pub mod artifact;
pub mod audit;
pub mod auth;
pub mod chat;
pub mod config;
pub mod health;
pub mod knowledge;
pub mod mcp;
pub mod memory;
pub mod metrics;
pub mod models;
pub mod saml;
pub mod scim;
pub mod spa;
pub mod stream;

use axum::routing::get;
use axum::Router;

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let router = Router::new()
        // Health & Diagnostics
        .route("/health", get(health::health_check))
        .route("/health/db", get(health::db_health))
        .route("/api/system/doctor", get(config::get_system_doctor))
        // Dynamic Model Discovery & Gateway Catalog
        .route("/api/models", get(models::list_models))
        .route("/api/v1/models", get(models::list_models))
        // Runtime Branding & Zero-Fork Configuration
        .route("/api/config/branding", get(config::get_branding_config))
        // Prometheus Metrics & FinOps Telemetry
        .route("/metrics", get(metrics::get_metrics))
        .route("/api/finops/summary", get(metrics::get_finops_summary))
        // Auth, Identity & Governance
        .route("/api/auth/me", get(auth::auth_me))
        .route("/api/auth/login", axum::routing::post(auth::login))
        .route("/api/auth/logout", axum::routing::post(auth::logout))
        .route("/api/auth/password", axum::routing::post(auth::change_password))
        .route("/api/admin/roles", get(auth::list_roles))
        .route("/api/admin/users", get(admin_users::list_admin_users))
        .route("/api/admin/users/role", axum::routing::post(admin_users::update_user_role))
        // Enterprise SAML 2.0 / rIDM BPS Identity Federation
        .route("/api/auth/sso", get(saml::saml_login))
        .route("/api/auth/saml/login", get(saml::saml_login))
        .route("/api/auth/saml/config", get(saml::saml_config))
        .route("/api/auth/saml/metadata", get(saml::saml_metadata))
        .route("/saml/metadata", get(saml::saml_metadata))
        .route("/api/auth/saml/acs", axum::routing::post(saml::saml_acs).get(saml::saml_acs))
        // Chat & Keyset Pagination
        .route(
            "/api/sessions",
            get(chat::list_sessions).post(chat::create_session),
        )
        .route(
            "/api/sessions/{session_id}/messages",
            get(chat::list_messages).post(chat::send_message),
        )
        // Durable User Memory System (Long-Term Semantic & Entity Memory)
        .route(
            "/api/user/memories",
            get(memory::list_memories).post(memory::create_memory),
        )
        .route(
            "/api/user/memories/{id}",
            axum::routing::delete(memory::delete_memory),
        )
        // Interactive Artifacts & Sandboxed PostMessage IPC (Phase 13)
        .route("/api/artifacts/parse", axum::routing::post(artifact::parse_artifacts))
        .route("/api/artifacts/ipc", axum::routing::post(artifact::handle_artifact_ipc))
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

        // Human-in-the-Loop Tool Execution Approvals (Phase 10)
        .route("/api/mcp/approvals/pending", get(approval::list_pending_approvals))
        .route("/api/mcp/approvals/{token}", get(approval::get_approval))
        .route("/api/mcp/approvals/{token}/approve", axum::routing::post(approval::approve_request))
        .route("/api/mcp/approvals/{token}/deny", axum::routing::post(approval::deny_request))

        // SCIM 2.0 Identity Governance & Automated Group Provisioning (Phase 9 - RFC 7644)
        .route("/scim/v2/Users", get(scim::list_scim_users).post(scim::create_scim_user))
        .route(
            "/scim/v2/Users/{id}",
            get(scim::get_scim_user).patch(scim::patch_scim_user).delete(scim::delete_scim_user),
        )
        .route("/scim/v2/Groups", get(scim::list_scim_groups).post(scim::create_scim_group))
        .route(
            "/scim/v2/Groups/{id}",
            get(scim::get_scim_group).delete(scim::delete_scim_group),
        )

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
    let static_path = if std::path::Path::new(&static_dir).exists() {
        Some(std::path::PathBuf::from(static_dir))
    } else if std::path::Path::new(&format!("../../{}", static_dir)).exists() {
        Some(std::path::PathBuf::from(format!("../../{}", static_dir)))
    } else {
        None
    };

    if let Some(path) = static_path {
        router.fallback_service(spa::create_spa_service(path))
    } else {
        router
    }
}

