use serde::{Deserialize, Serialize};
use crate::models::TokenUsageRecord;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetrySeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryDiagnosticEvent {
    pub title: String,
    pub message: String,
    pub severity: TelemetrySeverity,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub trace_id: Option<String>,
    pub error_type: Option<String>,
    pub attributes: serde_json::Value,
}

/// Universal Telemetry Exporter Contract.
/// Decouples Lumis from any specific observability vendor (Prometheus, Sentry, Datadog, OTel).
pub trait TelemetryExporter: Send + Sync {
    /// Canonical exporter name (e.g. "prometheus", "sentry", "otlp").
    fn exporter_name(&self) -> &str;

    /// Ingests a GenAI token usage and latency event.
    fn record_gen_ai_event(&self, record: &TokenUsageRecord);

    /// Captures a diagnostic event, breadcrumb, or error notification.
    fn capture_diagnostic(&self, event: &TelemetryDiagnosticEvent);

    /// Formats or flushes current metrics (e.g. for /metrics endpoint).
    fn export_metrics_text(&self) -> Option<String> {
        None
    }
}
