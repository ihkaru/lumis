#![deny(unsafe_code)]

use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::prompt_cache::{IdempotencyRecord, IdempotencyState};
use lumis_core::traits::idempotency::{IdempotencyClaimResult, IdempotencyStore};

pub struct InMemoryIdempotencyStore {
    records: RwLock<HashMap<String, IdempotencyRecord>>,
}

impl InMemoryIdempotencyStore {
    pub fn new() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryIdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdempotencyStore for InMemoryIdempotencyStore {
    async fn claim(
        &self,
        key: &str,
        user_id: &str,
        tenant_id: Option<&str>,
    ) -> Result<IdempotencyClaimResult> {
        let mut map = self.records.write().unwrap();
        if let Some(record) = map.get(key) {
            match &record.state {
                IdempotencyState::InFlight => Ok(IdempotencyClaimResult::InFlight),
                IdempotencyState::Completed(payload) => {
                    Ok(IdempotencyClaimResult::Completed(payload.clone()))
                }
            }
        } else {
            let record = IdempotencyRecord {
                key: key.to_string(),
                user_id: user_id.to_string(),
                tenant_id: tenant_id.map(|s| s.to_string()),
                created_at: Utc::now(),
                state: IdempotencyState::InFlight,
            };
            map.insert(key.to_string(), record);
            Ok(IdempotencyClaimResult::Acquired)
        }
    }

    async fn complete(&self, key: &str, response_payload: &str) -> Result<()> {
        let mut map = self.records.write().unwrap();
        if let Some(record) = map.get_mut(key) {
            record.state = IdempotencyState::Completed(response_payload.to_string());
            Ok(())
        } else {
            Err(LumisError::NotFound(format!("Idempotency key '{}' not found", key)))
        }
    }
}
