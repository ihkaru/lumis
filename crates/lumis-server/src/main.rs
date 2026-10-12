#![deny(unsafe_code)]

//! # Lumis Server
//! Production Axum 0.8 HTTP / SSE Server.
//! Pure async Rust, zero-copy token streaming, pluggable multi-database architecture.

use std::net::SocketAddr;
use std::sync::Arc;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes;
use lumis_server::state::AppState;
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

    // 4. Construct Application State
    let state = AppState::new(driver, guardrail, claim_translator, trusted_header);


    // 4. Build Axum Router
    let app = routes::create_router(state);

    // 5. Bind and Serve
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Lumis listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
