#![deny(unsafe_code)]

//! # Lumis Server Library
//! Core routing, authorization middleware, guardrails, and state.

pub mod finops;
pub mod guardrail;
pub mod llm;
pub mod rag;
pub mod routes;
pub mod state;
pub mod telemetry;

pub use finops::{InMemoryQuotaEngine, PrometheusTelemetryCollector};
pub use guardrail::SimdGuardrailMatcher;
pub use llm::{OpenAiCompatibleDriver, ResilientLlmRouter};
pub use rag::{DenseEmbedder, HybridKnowledgeVault, TextChunker};
pub use routes::create_router;
pub use state::AppState;
pub use telemetry::{CompositeTelemetryDispatcher, PrometheusExporter, SentryTelemetryExporter};



