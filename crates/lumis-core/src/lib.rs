#![deny(unsafe_code)]

//! # Lumis Core
//! Domain models, error definitions, and interface-first repository abstractions.
//! Adheres to SOLID Dependency Inversion Principle (DIP).

pub mod error;
pub mod models;
pub mod traits;

pub use error::{LumisError, Result};
pub use models::{
    ChatChunk, ChatCompletionRequest, ChatMessagePayload, CursorPage, CursorQuery, EnterpriseRole,
    Message, MessageRole, ModelClearanceTier, NewMessage, NewSession, ProviderHealth,
    ProviderHealthStatus, QuotaCheckResult, Session, TokenUsageRecord, User, UserClaims,
};
pub use traits::{
    ChatRepository, ClaimTranslator, DefaultClaimTranslator, DriverHealth, GuardrailMatcher,
    GuardrailVerdict, LlmProviderDriver, RelationalDriver, SqlDialect, TelemetryCollector,
    TelemetryDiagnosticEvent, TelemetryExporter, TelemetrySeverity, TokenQuotaEngine,
};
