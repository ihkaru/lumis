use std::sync::Arc;
use bytes::Bytes;
use lumis_core::traits::{
    AuditLogger, ChatRepository, ClaimTranslator, GuardrailMatcher, IdempotencyStore,
    KnowledgeVault, LlmProviderDriver, McpRegistry, RelationalDriver, ScimProvisioner,
    TelemetryCollector, TokenQuotaEngine, ToolApprovalManager,
};
use tokio::sync::broadcast;

use crate::approval::InMemoryApprovalManager;
use crate::audit::InMemoryAuditLogger;
use crate::finops::{InMemoryQuotaEngine, PrometheusTelemetryCollector};
use crate::idempotency::InMemoryIdempotencyStore;
use crate::mcp::{EnterpriseHookPipeline, InMemoryMcpRegistry};
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
    pub hook_pipeline: Arc<EnterpriseHookPipeline>,
    pub idempotency_store: Arc<dyn IdempotencyStore>,
    pub scim_provisioner: Arc<dyn ScimProvisioner>,
    pub approval_manager: Arc<dyn ToolApprovalManager>,
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
        let hook_pipeline = Arc::new(EnterpriseHookPipeline::with_defaults(audit_logger.clone()));
        Self {
            driver,
            repo: repo.clone(),
            guardrail,
            claim_translator,
            quota_engine,
            telemetry,
            knowledge_vault,
            mcp_registry,
            audit_logger,
            hook_pipeline,
            idempotency_store: Arc::new(InMemoryIdempotencyStore::default()),
            scim_provisioner: Arc::new(crate::scim::DatabaseScimProvisioner::new(repo)),
            approval_manager: Arc::new(InMemoryApprovalManager::default()),
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

    pub fn with_scim_provisioner(mut self, provisioner: Arc<dyn ScimProvisioner>) -> Self {
        self.scim_provisioner = provisioner;
        self
    }

    pub fn with_approval_manager(mut self, manager: Arc<dyn ToolApprovalManager>) -> Self {
        self.approval_manager = manager;
        self
    }
}
