use std::sync::Arc;
use bytes::Bytes;
use lumis_core::traits::{ChatRepository, ClaimTranslator, GuardrailMatcher, RelationalDriver};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub driver: Arc<dyn RelationalDriver>,
    pub repo: Arc<dyn ChatRepository>,
    pub guardrail: Arc<dyn GuardrailMatcher>,
    pub claim_translator: Arc<dyn ClaimTranslator>,
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
        let repo = driver.repository();
        let (stream_tx, _) = broadcast::channel(1024);
        Self {
            driver,
            repo,
            guardrail,
            claim_translator,
            stream_tx,
            trusted_header,
        }
    }
}

