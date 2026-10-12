use std::sync::Arc;
use bytes::Bytes;
use lumis_core::traits::{ChatRepository, GuardrailMatcher};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn ChatRepository>,
    pub guardrail: Arc<dyn GuardrailMatcher>,
    pub stream_tx: broadcast::Sender<Bytes>,
    pub trusted_header: String,
}

impl AppState {
    pub fn new(
        repo: Arc<dyn ChatRepository>,
        guardrail: Arc<dyn GuardrailMatcher>,
        trusted_header: String,
    ) -> Self {
        let (stream_tx, _) = broadcast::channel(1024);
        Self {
            repo,
            guardrail,
            stream_tx,
            trusted_header,
        }
    }
}
