use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use std::time::{Duration, Instant};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{ChatChunk, ChatCompletionRequest, ChatMessagePayload};
use lumis_core::traits::LlmProviderDriver;
use serde_json::json;
use tokio::sync::mpsc;

pub struct ModelCircuitBreaker {
    failures: HashMap<String, (u32, Instant)>,
    threshold: u32,
    cooldown: Duration,
}

static CIRCUIT_BREAKER: LazyLock<RwLock<ModelCircuitBreaker>> = LazyLock::new(|| {
    RwLock::new(ModelCircuitBreaker {
        failures: HashMap::new(),
        threshold: 2,
        cooldown: Duration::from_secs(45),
    })
});

pub fn is_model_circuit_open(model: &str) -> bool {
    let cb = CIRCUIT_BREAKER.read().unwrap();
    if let Some((consecutive, last_fail)) = cb.failures.get(model) {
        if *consecutive >= cb.threshold && last_fail.elapsed() < cb.cooldown {
            return true;
        }
    }
    false
}

pub fn record_model_success(model: &str) {
    let mut cb = CIRCUIT_BREAKER.write().unwrap();
    if let Some(entry) = cb.failures.get_mut(model) {
        entry.0 = 0;
    }
}

pub fn record_model_failure(model: &str) {
    let mut cb = CIRCUIT_BREAKER.write().unwrap();
    let entry = cb.failures.entry(model.to_string()).or_insert((0, Instant::now()));
    entry.0 += 1;
    entry.1 = Instant::now();
}

pub fn get_model_display_name(model_id: &str) -> String {
    crate::llm::model_metadata::format_model_display_name(model_id)
}

pub fn resolve_model_cascade(primary_model: &str) -> Vec<String> {
    let mut cascade = vec![primary_model.to_string()];
    let default_model = std::env::var("DEFAULT_MODEL").unwrap_or_else(|_| "my9model-free".to_string());

    if let Ok(env_fallbacks) = std::env::var("DEFAULT_FALLBACK_MODELS") {
        for fb in env_fallbacks.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if !cascade.iter().any(|m| m == fb) {
                cascade.push(fb.to_string());
            }
        }
    }

    for suffix in &["-contributor", "-fast", "-smart", "-turbo", "-distill"] {
        if let Some(base) = primary_model.strip_suffix(suffix) {
            if !cascade.iter().any(|m| m == base) {
                cascade.push(base.to_string());
            }
        }
    }

    for fb in &["openrouter-free", default_model.as_str()] {
        if !cascade.iter().any(|m| m == *fb) {
            cascade.push(fb.to_string());
        }
    }

    cascade
}

pub fn get_ttft_duration(model: &str) -> Duration {
    let lower = model.to_lowercase();
    let is_reasoning = lower.contains("reasoning")
        || lower.contains("think")
        || lower.contains("thought")
        || lower.contains("r1")
        || lower.contains("o1")
        || lower.contains("o3")
        || lower.contains("spark")
        || lower.contains("contributor")
        || lower.contains("wonder")
        || lower.contains("qwq");

    if is_reasoning {
        let ms = std::env::var("TTFT_REASONING_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(15_000);
        Duration::from_millis(ms)
    } else {
        let ms = std::env::var("TTFT_DEFAULT_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10_000);
        Duration::from_millis(ms)
    }
}

pub struct ResilientStreamResult {
    pub actual_model: String,
    pub failover_from: Option<String>,
    pub first_chunk: ChatChunk,
    pub rx: mpsc::Receiver<Result<ChatChunk>>,
}

pub async fn execute_chat_with_watchdog(
    router: &Arc<dyn LlmProviderDriver>,
    stream_tx: &tokio::sync::broadcast::Sender<bytes::Bytes>,
    sess_id: &str,
    primary_model: &str,
    messages: &[ChatMessagePayload],
    tools: Option<Vec<serde_json::Value>>,
) -> Result<ResilientStreamResult> {
    let cascade = resolve_model_cascade(primary_model);
    let mut last_err = None;

    for (idx, candidate) in cascade.iter().enumerate() {
        if is_model_circuit_open(candidate) {
            tracing::warn!(
                "Circuit breaker OPEN for model '{}', fast-failing in 0ms to next fallback",
                candidate
            );
            continue;
        }

        let status_msg = format!("Menghubungkan ke {}...", get_model_display_name(candidate));
        let status_event = json!({
            "event": "status",
            "session_id": sess_id,
            "model_id": candidate,
            "message": status_msg,
        });
        let _ = stream_tx.send(bytes::Bytes::from(status_event.to_string()));

        let req = ChatCompletionRequest {
            model: candidate.clone(),
            messages: messages.to_vec(),
            temperature: Some(0.7),
            max_tokens: None,
            stream: true,
            tools: tools.clone(),
            tool_choice: None,
        };

        let connect_res = tokio::time::timeout(Duration::from_secs(30), router.complete_chat(&req)).await;
        let mut rx = match connect_res {
            Ok(Ok(receiver)) => receiver,
            Ok(Err(e)) => {
                tracing::warn!("Model '{}' failed complete_chat: {}", candidate, e);
                record_model_failure(candidate);
                last_err = Some(e.to_string());
                notify_failover(stream_tx, sess_id, candidate, cascade.get(idx + 1));
                continue;
            }
            Err(_) => {
                tracing::warn!("Model '{}' connection timed out after 8s", candidate);
                record_model_failure(candidate);
                last_err = Some("Connection timed out after 8s".to_string());
                notify_failover(stream_tx, sess_id, candidate, cascade.get(idx + 1));
                continue;
            }
        };

        let ttft = get_ttft_duration(candidate);
        let first_res = tokio::time::timeout(ttft, rx.recv()).await;
        let first_chunk = match first_res {
            Ok(Some(Ok(chunk))) => {
                if chunk.is_done && chunk.content.trim().is_empty() && chunk.tool_calls.is_none() {
                    tracing::warn!("Model '{}' returned empty completion without tokens. Failing over...", candidate);
                    record_model_failure(candidate);
                    last_err = Some("Empty token response from upstream".to_string());
                    notify_failover(stream_tx, sess_id, candidate, cascade.get(idx + 1));
                    continue;
                }
                chunk
            }
            Ok(Some(Err(e))) => {
                tracing::warn!("Model '{}' stream error on initial token: {}", candidate, e);
                record_model_failure(candidate);
                last_err = Some(e.to_string());
                notify_failover(stream_tx, sess_id, candidate, cascade.get(idx + 1));
                continue;
            }
            Ok(None) => {
                tracing::warn!("Model '{}' closed stream before emitting initial token", candidate);
                record_model_failure(candidate);
                last_err = Some("Stream closed prematurely".to_string());
                notify_failover(stream_tx, sess_id, candidate, cascade.get(idx + 1));
                continue;
            }
            Err(_) => {
                tracing::warn!("TTFT Watchdog triggered: Model '{}' stalled (> {:?}). Failing over...", candidate, ttft);
                record_model_failure(candidate);
                last_err = Some(format!("Time-to-first-token stalled (> {:?})", ttft));
                notify_failover(stream_tx, sess_id, candidate, cascade.get(idx + 1));
                continue;
            }
        };

        record_model_success(candidate);
        let failover_from = if candidate != primary_model {
            Some(primary_model.to_string())
        } else {
            None
        };

        return Ok(ResilientStreamResult {
            actual_model: candidate.clone(),
            failover_from,
            first_chunk,
            rx,
        });
    }

    Err(LumisError::Internal(format!(
        "Semua model upstream tidak merespons (Terakhir: {})",
        last_err.unwrap_or_else(|| "Semua model sibuk".to_string())
    )))
}

fn notify_failover(
    stream_tx: &tokio::sync::broadcast::Sender<bytes::Bytes>,
    sess_id: &str,
    failed_model: &str,
    next_model: Option<&String>,
) {
    if let Some(next) = next_model {
        let event = json!({
            "event": "model_failover",
            "session_id": sess_id,
            "from_model": failed_model,
            "from_model_name": get_model_display_name(failed_model),
            "to_model": next,
            "to_model_name": get_model_display_name(next),
            "reason": "Model tidak merespons (TTFT watchdog timeout)",
        });
        let _ = stream_tx.send(bytes::Bytes::from(event.to_string()));
    }
}

pub fn process_chunk_payload(
    chunk: &ChatChunk,
    turn_text: &mut String,
    native_tcs: &mut HashMap<usize, (Option<String>, Option<String>, String)>,
    stream_tx: &tokio::sync::broadcast::Sender<bytes::Bytes>,
    sess_id: &str,
    model: &str,
) {
    if let Some(ref tcs) = chunk.tool_calls {
        for tc in tcs {
            let entry = native_tcs.entry(tc.index).or_insert((None, None, String::new()));
            if tc.id.is_some() { entry.0 = tc.id.clone(); }
            if let Some(ref n) = tc.name {
                if !n.is_empty() {
                    let name_str = entry.1.get_or_insert_with(String::new);
                    name_str.push_str(n);
                }
            }
            if let Some(ref arg_delta) = tc.arguments { entry.2.push_str(arg_delta); }
        }
    }
    if !chunk.content.is_empty() {
        turn_text.push_str(&chunk.content);
        if !turn_text.contains("<tool_call") && native_tcs.is_empty() {
            let chunk_event = json!({
                "event": "chunk",
                "session_id": sess_id,
                "delta": chunk.content,
                "model_id": model,
                "is_done": false,
            });
            let _ = stream_tx.send(bytes::Bytes::from(chunk_event.to_string()));
        }
    }
}

pub async fn collect_stream_turn(
    first_chunk: ChatChunk,
    mut rx: mpsc::Receiver<Result<ChatChunk>>,
    stream_tx: &tokio::sync::broadcast::Sender<bytes::Bytes>,
    sess_id: &str,
    model: &str,
) -> (String, HashMap<usize, (Option<String>, Option<String>, String)>) {
    let mut turn_text = String::new();
    let mut native_tcs = HashMap::new();

    process_chunk_payload(&first_chunk, &mut turn_text, &mut native_tcs, stream_tx, sess_id, model);

    if !first_chunk.is_done {
        while let Some(chunk_res) = rx.recv().await {
            match chunk_res {
                Ok(chunk) => {
                    let done = chunk.is_done;
                    process_chunk_payload(&chunk, &mut turn_text, &mut native_tcs, stream_tx, sess_id, model);
                    if done { break; }
                }
                Err(e) => {
                    tracing::error!("Error receiving chunk: {}", e);
                    break;
                }
            }
        }
    }

    (turn_text, native_tcs)
}
