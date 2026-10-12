use std::sync::Arc;
use lumis_core::models::TokenUsageRecord;
use lumis_core::traits::{
    TelemetryCollector, TelemetryDiagnosticEvent, TelemetryExporter,
};

/// Composite Telemetry Dispatcher fanning out to multiple APM backends.
/// Guarantees zero observability vendor lock-in.
pub struct CompositeTelemetryDispatcher {
    exporters: Vec<Arc<dyn TelemetryExporter>>,
}

impl CompositeTelemetryDispatcher {
    pub fn new(exporters: Vec<Arc<dyn TelemetryExporter>>) -> Self {
        Self { exporters }
    }

    pub fn default_enterprise_suite(sentry_dsn: Option<String>) -> Self {
        let prometheus = Arc::new(crate::telemetry::prometheus_exporter::PrometheusExporter::new());
        let sentry = Arc::new(crate::telemetry::sentry_exporter::SentryTelemetryExporter::new(sentry_dsn));
        Self {
            exporters: vec![prometheus, sentry],
        }
    }

    pub fn record_gen_ai_event(&self, record: &TokenUsageRecord) {
        for exporter in &self.exporters {
            exporter.record_gen_ai_event(record);
        }
    }

    pub fn capture_diagnostic(&self, event: &TelemetryDiagnosticEvent) {
        for exporter in &self.exporters {
            exporter.capture_diagnostic(event);
        }
    }
}

impl TelemetryExporter for CompositeTelemetryDispatcher {
    fn exporter_name(&self) -> &str {
        "composite_multi_vendor"
    }

    fn record_gen_ai_event(&self, record: &TokenUsageRecord) {
        for exporter in &self.exporters {
            exporter.record_gen_ai_event(record);
        }
    }

    fn capture_diagnostic(&self, event: &TelemetryDiagnosticEvent) {
        for exporter in &self.exporters {
            exporter.capture_diagnostic(event);
        }
    }

    fn export_metrics_text(&self) -> Option<String> {
        for exporter in &self.exporters {
            if let Some(text) = exporter.export_metrics_text() {
                return Some(text);
            }
        }
        None
    }
}

impl TelemetryCollector for CompositeTelemetryDispatcher {
    fn record_gen_ai_event(&self, record: &TokenUsageRecord) {
        for exporter in &self.exporters {
            exporter.record_gen_ai_event(record);
        }
    }

    fn render_prometheus_metrics(&self) -> String {
        self.export_metrics_text().unwrap_or_default()
    }
}
