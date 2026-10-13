#![deny(unsafe_code)]

//! # Lumis Core
//! Domain models, error definitions, and interface-first repository abstractions.
//! Adheres to SOLID Dependency Inversion Principle (DIP).

pub mod error;
pub mod models;
pub mod security;
pub mod traits;

pub use error::{LumisError, Result};
pub use models::{
    ArtifactIpcAction, ArtifactIpcRequest, ArtifactIpcResponse, ArtifactParser,
    ArtifactSpecification, ArtifactType, AuditEventStatus, AuditLogEvent, ChatChunk,
    ChatCompletionRequest, ChatMessagePayload, CursorPage, CursorQuery, Document,
    DocumentChunk, EnterpriseRole, KnowledgeQuery, KnowledgeQueryResult, McpAuthType,
    McpRiskTier, McpServerCreateInput, McpServerRecord, McpServerUpdateInput, McpTaskState,
    McpTaskStatus, McpToolCallRequest, McpToolCallResult, McpToolDefinition,
    McpToolInvocationRecord, McpToolRecord, McpTransportType, Message, MessageRole,
    ModelClearanceTier, NewMessage, NewSession, ProviderHealth, ProviderHealthStatus,
    QuotaCheckResult, Session, TokenUsageRecord, User, UserClaims,
};
pub use traits::{
    AuditLogger, ChatRepository, ClaimTranslator, DefaultClaimTranslator, DriverHealth,
    GuardrailMatcher, GuardrailVerdict, KnowledgeVault, LlmProviderDriver, McpRegistry,
    McpToolExecutor, RelationalDriver, SqlDialect, TelemetryCollector, TelemetryDiagnosticEvent,
    TelemetryExporter, TelemetrySeverity, TokenQuotaEngine,
};

