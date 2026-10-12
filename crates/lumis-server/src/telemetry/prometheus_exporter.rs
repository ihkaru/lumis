use lumis_core::models::TokenUsageRecord;
use lumis_core::traits::{TelemetryDiagnosticEvent, TelemetryExporter};
use crate::finops::PrometheusTelemetryCollector;

/// Prometheus Telemetry Exporter implementing the universal TelemetryExporter contract.
pub struct PrometheusExporter {
    collector: PrometheusTelemetryCollector,
}

impl PrometheusExporter {
    pub fn new() -> Self {
        Self {
            collector: PrometheusTelemetryCollector::new(),
        }
    }
}

impl Default for PrometheusExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryExporter for PrometheusExporter {
    fn exporter_name(&self) -> &str {
        "prometheus"
    }

    fn record_gen_ai_event(&self, record: &TokenUsageRecord) {
        use lumis_core::traits::TelemetryCollector;
        self.collector.record_gen_ai_event(record);
    }

    fn capture_diagnostic(&self, _event: &TelemetryDiagnosticEvent) {
        // Prometheus primarily focuses on time-series metrics.
    }

    fn export_metrics_text(&self) -> Option<String> {
        use lumis_core::traits::TelemetryCollector;
        Some(self.collector.render_prometheus_metrics())
    }
}
