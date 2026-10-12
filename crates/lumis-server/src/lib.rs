#![deny(unsafe_code)]

//! # Lumis Server Library
//! Core routing, authorization middleware, guardrails, and state.

pub mod finops;
pub mod guardrail;
pub mod routes;
pub mod state;

pub use finops::{InMemoryQuotaEngine, PrometheusTelemetryCollector};
pub use guardrail::SimdGuardrailMatcher;
pub use routes::create_router;
pub use state::AppState;

