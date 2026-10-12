use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{ChatChunk, ChatCompletionRequest, ProviderHealth, ProviderHealthStatus};
use lumis_core::traits::{LlmProviderDriver, TelemetryDiagnosticEvent, TelemetryExporter, TelemetrySeverity};
use tokio::sync::mpsc;

struct CircuitBreakerState {
    consecutive_failures: u32,
    last_failure: Option<Instant>,
}

/// Enterprise High-Availability Resilient LLM Router.
/// Manages automated failover across multiple AI providers with Circuit Breaker protection.
pub struct ResilientLlmRouter {
    providers: Vec<Arc<dyn LlmProviderDriver>>,
    circuit_states: RwLock<HashMap<String, CircuitBreakerState>>,
    failure_threshold: u32,
    cooldown_duration: Duration,
    telemetry: Option<Arc<dyn TelemetryExporter>>,
}

impl ResilientLlmRouter {
    pub fn new(
        primary: Arc<dyn LlmProviderDriver>,
        fallbacks: Vec<Arc<dyn LlmProviderDriver>>,
        telemetry: Option<Arc<dyn TelemetryExporter>>,
    ) -> Self {
        let mut providers = vec![primary];
        providers.extend(fallbacks);

        Self {
            providers,
            circuit_states: RwLock::new(HashMap::new()),
            failure_threshold: 3,
            cooldown_duration: Duration::from_secs(30),
            telemetry,
        }
    }

    fn is_circuit_open(&self, provider_name: &str) -> bool {
        let states = self.circuit_states.read().unwrap();
        if let Some(state) = states.get(provider_name) {
            if state.consecutive_failures >= self.failure_threshold {
                if let Some(last_fail) = state.last_failure {
                    return last_fail.elapsed() < self.cooldown_duration;
                }
            }
        }
        false
    }

    fn record_success(&self, provider_name: &str) {
        let mut states = self.circuit_states.write().unwrap();
        if let Some(state) = states.get_mut(provider_name) {
            state.consecutive_failures = 0;
            state.last_failure = None;
        }
    }

    fn record_failure(&self, provider_name: &str, error_msg: &str) {
        let mut states = self.circuit_states.write().unwrap();
        let state = states.entry(provider_name.to_string()).or_insert(CircuitBreakerState {
            consecutive_failures: 0,
            last_failure: None,
        });
        state.consecutive_failures += 1;
        state.last_failure = Some(Instant::now());

        if let Some(ref tel) = self.telemetry {
            tel.capture_diagnostic(&TelemetryDiagnosticEvent {
                title: "LLM Provider Circuit Breaker Failure Recorded".to_string(),
                message: format!("Provider '{}' failed: {}", provider_name, error_msg),
                severity: TelemetrySeverity::Warning,
                provider: Some(provider_name.to_string()),
                model: None,
                trace_id: None,
                error_type: Some("provider_error".to_string()),
                attributes: serde_json::json!({
                    "consecutive_failures": state.consecutive_failures,
                    "circuit_open": state.consecutive_failures >= self.failure_threshold,
                }),
            });
        }
    }
}

#[async_trait]
impl LlmProviderDriver for ResilientLlmRouter {
    fn provider_name(&self) -> &str {
        "resilient_multi_provider_router"
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        let mut statuses = Vec::new();
        for provider in &self.providers {
            if let Ok(health) = provider.health_check().await {
                statuses.push(format!("{}: {:?}", provider.provider_name(), health.status));
            }
        }
        Ok(ProviderHealth {
            provider_name: self.provider_name().to_string(),
            status: ProviderHealthStatus::Healthy,
            latency_ms: 0,
        })
    }

    async fn complete_chat(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<mpsc::Receiver<Result<ChatChunk>>> {
        let mut last_err = None;

        for (idx, provider) in self.providers.iter().enumerate() {
            let name = provider.provider_name();

            if self.is_circuit_open(name) {
                tracing::warn!("Circuit open for provider '{}', skipping to fallback", name);
                continue;
            }

            match provider.complete_chat(request).await {
                Ok(rx) => {
                    self.record_success(name);
                    if idx > 0 {
                        tracing::info!("Failover successfully resolved by backup provider '{}'", name);
                        if let Some(ref tel) = self.telemetry {
                            tel.capture_diagnostic(&TelemetryDiagnosticEvent {
                                title: "LLM SLA Failover Resolved".to_string(),
                                message: format!("Request successfully fulfilled by fallback provider '{}'", name),
                                severity: TelemetrySeverity::Info,
                                provider: Some(name.to_string()),
                                model: Some(request.model.clone()),
                                trace_id: None,
                                error_type: None,
                                attributes: serde_json::json!({ "fallback_index": idx }),
                            });
                        }
                    }
                    return Ok(rx);
                }
                Err(e) => {
                    tracing::warn!("Provider '{}' failed with error: {}. Attempting next fallback...", name, e);
                    self.record_failure(name, &e.to_string());
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            LumisError::Internal("All configured LLM providers in resilient pool are currently unavailable".to_string())
        }))
    }
}
