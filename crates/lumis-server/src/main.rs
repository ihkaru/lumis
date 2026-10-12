#![deny(unsafe_code)]

//! # Lumis Server
//! Production Axum 0.8 HTTP / SSE Server.
//! Pure async Rust, zero-copy token streaming, pluggable multi-database architecture,
//! multi-provider resilient LLM router, and vendor-agnostic telemetry (OTel, Prometheus, Sentry).

use std::net::SocketAddr;
use std::sync::Arc;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::llm::{OpenAiCompatibleDriver, ResilientLlmRouter};
use lumis_server::routes;
use lumis_server::state::AppState;
use lumis_server::telemetry::CompositeTelemetryDispatcher;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lumis_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://lumis.sqlite?mode=rwc".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);
    let trusted_header = std::env::var("TRUSTED_HEADER")
        .unwrap_or_else(|_| "x-forwarded-user".to_string());

    tracing::info!("Starting Lumis Enterprise Server on port {}", port);

    // 1. Initialize Pluggable Relational Driver (EIP Channel Adapter & DIP)
    let driver = DatabaseFactory::create_driver(&database_url).await?;
    driver.run_migrations().await?;

    // 2. Initialize SIMD Guardrail Matcher
    let guardrail = Arc::new(SimdGuardrailMatcher::new(
        SimdGuardrailMatcher::default_patterns(),
    )?);

    // 3. Initialize IAM Claims Translator (EIP Message Translator)
    let claim_translator = Arc::new(lumis_core::traits::DefaultClaimTranslator::new());

    // 4. Initialize Multi-Vendor Telemetry Suite (Prometheus + Sentry + OTel)
    let sentry_dsn = std::env::var("SENTRY_DSN").ok();
    let telemetry = Arc::new(CompositeTelemetryDispatcher::default_enterprise_suite(sentry_dsn));
    let quota_engine = Arc::new(lumis_server::finops::InMemoryQuotaEngine::new());

    // 5. Construct Application State with Vendor-Agnostic Telemetry
    let mut state = AppState::with_options(
        driver,
        guardrail,
        claim_translator,
        quota_engine,
        telemetry.clone(),
        trusted_header,
    );

    // 6. Optional: Initialize Resilient Multi-Provider LLM Router (if upstream URLs configured)
    let primary_llm_url = std::env::var("LLM_API_BASE")
        .or_else(|_| std::env::var("OPENAI_API_BASE"))
        .ok();

    if let Some(primary_url) = primary_llm_url {
        let primary_key = std::env::var("LLM_API_KEY").or_else(|_| std::env::var("OPENAI_API_KEY")).ok();
        let primary_driver = Arc::new(OpenAiCompatibleDriver::new(
            "primary_upstream".to_string(),
            primary_url,
            primary_key,
        ));

        let backup_drivers: Vec<Arc<dyn lumis_core::traits::LlmProviderDriver>> = match std::env::var("LLM_BACKUP_API_BASE") {
            Ok(backup_url) => {
                let backup_key = std::env::var("LLM_BACKUP_API_KEY").ok();
                vec![Arc::new(OpenAiCompatibleDriver::new(
                    "backup_upstream".to_string(),
                    backup_url,
                    backup_key,
                ))]
            }
            Err(_) => Vec::new(),
        };

        let router = Arc::new(ResilientLlmRouter::new(
            primary_driver,
            backup_drivers,
            Some(telemetry.clone()),
        ));

        state = state.with_llm_router(router);
        tracing::info!("Resilient Multi-Provider LLM Router initialized");
    }

    // 7. Build Axum Router
    let app = routes::create_router(state);

    // 8. Bind and Serve
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Lumis listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
