use std::collections::HashMap;
use std::sync::RwLock;
use lumis_core::models::TokenUsageRecord;
use lumis_core::traits::TelemetryCollector;

/// OpenTelemetry GenAI Metric Collector rendering Prometheus format.
pub struct PrometheusTelemetryCollector {
    token_usage_counters: RwLock<HashMap<String, u64>>,
    duration_ms_sum: RwLock<HashMap<String, u64>>,
    operation_counts: RwLock<HashMap<String, u64>>,
}

impl PrometheusTelemetryCollector {
    pub fn new() -> Self {
        Self {
            token_usage_counters: RwLock::new(HashMap::new()),
            duration_ms_sum: RwLock::new(HashMap::new()),
            operation_counts: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for PrometheusTelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryCollector for PrometheusTelemetryCollector {
    fn record_gen_ai_event(&self, record: &TokenUsageRecord) {
        let input_key = format!(
            "provider=\"{}\",model=\"{}\",role=\"{}\",type=\"input\"",
            record.provider,
            record.model_id,
            record.role.as_str()
        );
        let output_key = format!(
            "provider=\"{}\",model=\"{}\",role=\"{}\",type=\"output\"",
            record.provider,
            record.model_id,
            record.role.as_str()
        );
        let op_key = format!("provider=\"{}\",model=\"{}\"", record.provider, record.model_id);

        {
            let mut tokens = self.token_usage_counters.write().unwrap();
            *tokens.entry(input_key).or_insert(0) += record.input_tokens;
            *tokens.entry(output_key).or_insert(0) += record.output_tokens;
        }

        {
            let mut durations = self.duration_ms_sum.write().unwrap();
            *durations.entry(op_key.clone()).or_insert(0) += record.duration_ms;
        }

        {
            let mut ops = self.operation_counts.write().unwrap();
            *ops.entry(op_key).or_insert(0) += 1;
        }
    }

    fn render_prometheus_metrics(&self) -> String {
        let mut out = String::with_capacity(2048);

        out.push_str("# HELP gen_ai_client_token_usage_total OpenTelemetry GenAI token usage by provider, model, and role\n");
        out.push_str("# TYPE gen_ai_client_token_usage_total counter\n");
        {
            let tokens = self.token_usage_counters.read().unwrap();
            for (labels, count) in tokens.iter() {
                out.push_str(&format!("gen_ai_client_token_usage_total{{{}}} {}\n", labels, count));
            }
        }

        out.push_str("\n# HELP gen_ai_client_operation_duration_ms_total Total duration of GenAI operations in ms\n");
        out.push_str("# TYPE gen_ai_client_operation_duration_ms_total counter\n");
        {
            let durations = self.duration_ms_sum.read().unwrap();
            for (labels, sum) in durations.iter() {
                out.push_str(&format!("gen_ai_client_operation_duration_ms_total{{{}}} {}\n", labels, sum));
            }
        }

        out.push_str("\n# HELP gen_ai_client_operations_total Total count of GenAI operations\n");
        out.push_str("# TYPE gen_ai_client_operations_total counter\n");
        {
            let ops = self.operation_counts.read().unwrap();
            for (labels, count) in ops.iter() {
                out.push_str(&format!("gen_ai_client_operations_total{{{}}} {}\n", labels, count));
            }
        }

        out
    }
}
