use async_trait::async_trait;

use crate::error::LumisError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdempotencyClaimResult {
    /// Request is new, claim acquired. Safe to proceed with rate limiter and generation.
    Acquired,
    /// Request with this key is currently in-flight. Caller must attach or wait, with 0 quota penalty.
    InFlight,
    /// Request previously completed. Returns the cached payload directly with 0 quota penalty.
    Completed(String),
}

#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    async fn claim(
        &self,
        key: &str,
        user_id: &str,
        tenant_id: Option<&str>,
    ) -> Result<IdempotencyClaimResult, LumisError>;

    async fn complete(
        &self,
        key: &str,
        response_payload: &str,
    ) -> Result<(), LumisError>;
}
