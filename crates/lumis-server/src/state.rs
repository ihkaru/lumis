use std::sync::Arc;
use bytes::Bytes;
use lumis_core::traits::{
    ChatRepository, ClaimTranslator, GuardrailMatcher, KnowledgeVault, LlmProviderDriver,
    RelationalDriver, TelemetryCollector, TokenQuotaEngine,
};
use tokio::sync::broadcast;

use crate::finops::{InMemoryQuotaEngine, PrometheusTelemetryCollector};
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
            llm_router: None,
            stream_tx,
            trusted_header,
        }
    }

    pub fn with_llm_router(mut self, router: Arc<dyn LlmProviderDriver>) -> Self {
        self.llm_router = Some(router);
        self
    }
}
