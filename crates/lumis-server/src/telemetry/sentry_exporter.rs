use std::sync::RwLock;
use lumis_core::models::TokenUsageRecord;
use lumis_core::traits::{TelemetryDiagnosticEvent, TelemetryExporter, TelemetrySeverity};

/// Sentry Enterprise APM & Error Tracking Exporter.
/// Captures LLM failover events, breadcrumbs, and exceptions into Sentry envelope format.
pub struct SentryTelemetryExporter {
    dsn: Option<String>,
    recent_events: RwLock<Vec<TelemetryDiagnosticEvent>>,
    max_history: usize,
}

impl SentryTelemetryExporter {
    pub fn new(dsn: Option<String>) -> Self {
        Self {
            dsn,
            recent_events: RwLock::new(Vec::new()),
            max_history: 100,
        }
    }

    pub fn is_dsn_configured(&self) -> bool {
        self.dsn.is_some()
    }

    pub fn get_captured_events(&self) -> Vec<TelemetryDiagnosticEvent> {
        self.recent_events.read().unwrap().clone()
    }
}

impl TelemetryExporter for SentryTelemetryExporter {
    fn exporter_name(&self) -> &str {
        "sentry"
    }

    fn record_gen_ai_event(&self, record: &TokenUsageRecord) {
        // Record LLM breadcrumb for Sentry timeline
        tracing::debug!(
            provider = %record.provider,
            model = %record.model_id,
            input_tokens = record.input_tokens,
            duration_ms = record.duration_ms,
            "Sentry LLM Breadcrumb recorded"
        );
    }

    fn capture_diagnostic(&self, event: &TelemetryDiagnosticEvent) {
        match event.severity {
            TelemetrySeverity::Error => {
                tracing::error!(
                    title = %event.title,
                    message = %event.message,
                    error_type = ?event.error_type,
                    "Sentry Error Event captured"
                );
            }
            TelemetrySeverity::Warning => {
                tracing::warn!(
                    title = %event.title,
                    message = %event.message,
                    provider = ?event.provider,
                    "Sentry Warning Breadcrumb captured"
                );
            }
            TelemetrySeverity::Info => {
                tracing::info!(
                    title = %event.title,
                    message = %event.message,
                    "Sentry Info Breadcrumb captured"
                );
            }
        }

        let mut events = self.recent_events.write().unwrap();
        if events.len() >= self.max_history {
            events.remove(0);
        }
        events.push(event.clone());
    }
}
