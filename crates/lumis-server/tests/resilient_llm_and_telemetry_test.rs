use std::sync::Arc;
use async_trait::async_trait;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{
    ChatChunk, ChatCompletionRequest, ChatMessagePayload, EnterpriseRole, ProviderHealth,
    ProviderHealthStatus, TokenUsageRecord,
};
use lumis_core::traits::{
    LlmProviderDriver, TelemetryCollector, TelemetryDiagnosticEvent,
    TelemetrySeverity,
};
use lumis_server::llm::ResilientLlmRouter;
use lumis_server::telemetry::{
    CompositeTelemetryDispatcher, PrometheusExporter, SentryTelemetryExporter,
};
use tokio::sync::mpsc;

struct FailingTestDriver {
    name: String,
}

#[async_trait]
impl LlmProviderDriver for FailingTestDriver {
    fn provider_name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        Ok(ProviderHealth {
            provider_name: self.name.clone(),
            status: ProviderHealthStatus::Offline("Simulated 503 Provider Outage".to_string()),
            latency_ms: 150,
        })
    }

    async fn complete_chat(
        &self,
        _request: &ChatCompletionRequest,
    ) -> Result<mpsc::Receiver<Result<ChatChunk>>> {
        Err(LumisError::Internal("Upstream 503 Service Unavailable / Rate Limit 429".to_string()))
    }
}

struct HealthyTestDriver {
    name: String,
    emitted_text: String,
}

#[async_trait]
impl LlmProviderDriver for HealthyTestDriver {
    fn provider_name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        Ok(ProviderHealth {
            provider_name: self.name.clone(),
            status: ProviderHealthStatus::Healthy,
            latency_ms: 25,
        })
    }

    async fn complete_chat(
        &self,
        _request: &ChatCompletionRequest,
    ) -> Result<mpsc::Receiver<Result<ChatChunk>>> {
        let (tx, rx) = mpsc::channel(4);
        let text = self.emitted_text.clone();
        tokio::spawn(async move {
            let _ = tx.send(Ok(ChatChunk {
                content: text,
                is_done: false,
                model: Some("vllm-deepseek".to_string()),
                tool_calls: None,
            })).await;
            let _ = tx.send(Ok(ChatChunk::new("", true))).await;
        });
        Ok(rx)
    }
}

#[tokio::test]
async fn test_resilient_llm_router_automatic_failover_to_vllm() {
    let sentry_exporter = Arc::new(SentryTelemetryExporter::new(Some("https://key@sentry.io/123".to_string())));
    let prometheus_exporter = Arc::new(PrometheusExporter::new());
    let composite_telemetry = Arc::new(CompositeTelemetryDispatcher::new(vec![
        prometheus_exporter.clone(),
        sentry_exporter.clone(),
    ]));

    // Primary: Cloud Provider (Fails with 503 / 429)
    let primary = Arc::new(FailingTestDriver {
        name: "azure_openai_primary".to_string(),
    });

    // Secondary: On-premises private vLLM (Succeeds)
    let fallback = Arc::new(HealthyTestDriver {
        name: "vllm_on_premise_backup".to_string(),
        emitted_text: "Respons dari private vLLM cadangan.".to_string(),
    });

    let router = ResilientLlmRouter::new(
        primary,
        vec![fallback],
        Some(composite_telemetry.clone()),
    );

    let req = ChatCompletionRequest::new(
        "gpt-4o",
        vec![ChatMessagePayload::new("user", "Tes reliabilitas sistem")],
    );

    // Execute chat: Should automatically fail over to secondary vLLM
    let mut rx = router.complete_chat(&req).await.expect("Failover succeeded");
    let mut collected = String::new();
    while let Some(chunk_res) = rx.recv().await {
        let chunk = chunk_res.expect("Valid chunk");
        collected.push_str(&chunk.content);
        if chunk.is_done {
            break;
        }
    }

    assert_eq!(collected, "Respons dari private vLLM cadangan.");

    // Verify Sentry captured the failover warning and resolved event
    let captured = sentry_exporter.get_captured_events();
    assert!(!captured.is_empty());
    let has_failover_warning = captured.iter().any(|e| e.severity == TelemetrySeverity::Warning);
    let has_resolved_info = captured.iter().any(|e| e.severity == TelemetrySeverity::Info);
    assert!(has_failover_warning, "Expected failover warning recorded in Sentry");
    assert!(has_resolved_info, "Expected failover resolved event in Sentry");
}

#[tokio::test]
async fn test_multi_vendor_telemetry_fanout_without_lockin() {
    let sentry = Arc::new(SentryTelemetryExporter::new(None));
    let prometheus = Arc::new(PrometheusExporter::new());
    let composite = CompositeTelemetryDispatcher::new(vec![prometheus.clone(), sentry.clone()]);

    let usage = TokenUsageRecord {
        tenant_id: "satker_01".to_string(),
        user_id: "analyst_1".to_string(),
        role: EnterpriseRole::Member,
        model_id: "deepseek-v3".to_string(),
        provider: "vllm".to_string(),
        input_tokens: 350,
        output_tokens: 120,
        duration_ms: 45,
        timestamp: chrono::Utc::now(),
    };

    // Fan-out to all telemetry vendors
    composite.record_gen_ai_event(&usage);

    // Verify Prometheus text export
    let prom_text = composite.render_prometheus_metrics();
    assert!(prom_text.contains("gen_ai_client_token_usage_total"));
    assert!(prom_text.contains("provider=\"vllm\""));

    // Verify Sentry captures diagnostic event without vendor lock
    composite.capture_diagnostic(&TelemetryDiagnosticEvent {
        title: "Test Alert".to_string(),
        message: "Vendor-agnostic test message".to_string(),
        severity: TelemetrySeverity::Error,
        provider: Some("vllm".to_string()),
        model: Some("deepseek-v3".to_string()),
        trace_id: None,
        error_type: Some("io_timeout".to_string()),
        attributes: serde_json::json!({}),
    });

    let sentry_events = sentry.get_captured_events();
    assert_eq!(sentry_events.len(), 1);
    assert_eq!(sentry_events[0].title, "Test Alert");
}
