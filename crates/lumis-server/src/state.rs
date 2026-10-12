use std::sync::Arc;
use bytes::Bytes;
use lumis_core::traits::{
    AuditLogger, ChatRepository, ClaimTranslator, GuardrailMatcher, IdempotencyStore,
    KnowledgeVault, LlmProviderDriver, McpRegistry, RelationalDriver, TelemetryCollector,
    TokenQuotaEngine,
};
use tokio::sync::broadcast;

use crate::audit::InMemoryAuditLogger;
use crate::finops::{InMemoryQuotaEngine, PrometheusTelemetryCollector};
use crate::idempotency::InMemoryIdempotencyStore;
use crate::mcp::InMemoryMcpRegistry;
use crate::rag::HybridKnowledgeVault;

#[derive(Clone)]
pub struct AppState {
    pub driver: Arc<dyn RelationalDriver>,
    pub repo: Arc<dyn ChatRepository>,
    pub guardrail: Arc<dyn GuardrailMatcher>,
    pub claim_translator: Arc<dyn ClaimTranslator>,
    pub quota_engine: Arc<dyn TokenQuotaEngine>,
    pub telemetry: Arc<dyn TelemetryCollector>,
    pub knowledge_vault: Arc<dyn KnowledgeVault>,
    pub mcp_registry: Arc<dyn McpRegistry>,
    pub audit_logger: Arc<dyn AuditLogger>,
    pub idempotency_store: Arc<dyn IdempotencyStore>,
    pub llm_router: Option<Arc<dyn LlmProviderDriver>>,
    pub stream_tx: broadcast::Sender<Bytes>,
    pub trusted_header: String,
}

impl AppState {
    pub fn new(
        driver: Arc<dyn RelationalDriver>,
        guardrail: Arc<dyn GuardrailMatcher>,
        claim_translator: Arc<dyn ClaimTranslator>,
        trusted_header: String,
    ) -> Self {
        Self::with_options(
            driver,
            guardrail,
            claim_translator,
            Arc::new(InMemoryQuotaEngine::new()),
            Arc::new(PrometheusTelemetryCollector::new()),
            Arc::new(HybridKnowledgeVault::new()),
            Arc::new(InMemoryMcpRegistry::default()),
            Arc::new(InMemoryAuditLogger::default()),
            trusted_header,
        )
    }

    pub fn with_options(
        driver: Arc<dyn RelationalDriver>,
        guardrail: Arc<dyn GuardrailMatcher>,
        claim_translator: Arc<dyn ClaimTranslator>,
        quota_engine: Arc<dyn TokenQuotaEngine>,
        telemetry: Arc<dyn TelemetryCollector>,
        knowledge_vault: Arc<dyn KnowledgeVault>,
        mcp_registry: Arc<dyn McpRegistry>,
        audit_logger: Arc<dyn AuditLogger>,
        trusted_header: String,
    ) -> Self {
        let repo = driver.repository();
        let (stream_tx, _) = broadcast::channel(1024);
        Self {
            driver,
            repo,
            guardrail,
            claim_translator,
            quota_engine,
            telemetry,
            knowledge_vault,
            mcp_registry,
            audit_logger,
            idempotency_store: Arc::new(InMemoryIdempotencyStore::default()),
            llm_router: None,
            stream_tx,
            trusted_header,
        }
    }

    pub fn with_llm_router(mut self, router: Arc<dyn LlmProviderDriver>) -> Self {
        self.llm_router = Some(router);
        self
    }

    pub fn with_idempotency_store(mut self, store: Arc<dyn IdempotencyStore>) -> Self {
        self.idempotency_store = store;
        self
    }
}
