#![deny(unsafe_code)]

//! # Lumis Server Library
//! Core routing, authorization middleware, guardrails, and state.

pub mod guardrail;
pub mod routes;
pub mod state;

pub use guardrail::SimdGuardrailMatcher;
pub use routes::create_router;
pub use state::AppState;
