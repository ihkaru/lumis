use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use lumis_core::models::{CursorQuery, MessageRole, NewMessage};

use crate::state::AppState;
use super::helpers::resolve_claims_and_role;
use super::types::{ErrorResponse, PaginationQuery, SendMessageRequest};

pub async fn list_messages(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<PaginationQuery>,
) -> impl IntoResponse {
    let query = CursorQuery {
        cursor_time: params.cursor_time,
        cursor_id: params.cursor_id,
        limit: params.limit.unwrap_or(20),
    };

    match state.repo.list_messages(&session_id, &query).await {
        Ok(page) => (StatusCode::OK, Json(page)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn send_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Json(payload): Json<SendMessageRequest>,
) -> impl IntoResponse {
    let (username, role) = resolve_claims_and_role(&state, &headers);
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default");

    // 0. SOC Emergency Killswitch Enforcement
    if state.audit_logger.is_killswitch_active() {
        let (_, reason, _) = state.audit_logger.get_killswitch_status();
        let reason_msg = reason.unwrap_or_else(|| "Emergency security lockdown active".to_string());
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: format!("Global Emergency Killswitch Active: {}", reason_msg),
            }),
        )
            .into_response();
    }

    // 1. Policy Enforcement Point (PEP): Model Clearance verification
    if let Some(ref model_id) = payload.model_id {
        if let Err(e) = state.claim_translator.evaluate_model_clearance(role, model_id) {
            return (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: format!("Policy Enforcement Denied: {}", e),
                }),
            )
                .into_response();
        }
    }

    // 2. FinOps Token Budget Pre-Flight Check
    let estimated_tokens = ((payload.content.len() / 4).max(1)) as u64;
    match state
        .quota_engine
        .check_quota(tenant_id, &username, role, estimated_tokens)
        .await
    {
        lumis_core::models::QuotaCheckResult::Exceeded {
            limit,
            used,
            retry_after_secs,
        } => {
            let mut res = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": format!(
                        "FinOps Token Quota Exceeded: limit of {} tokens/hr exhausted for role '{}' (used: {}).",
                        limit,
                        role.as_str(),
                        used
                    ),
                    "role": role.as_str(),
                    "limit": limit,
                    "used": used,
                    "retry_after_secs": retry_after_secs,
                })),
            )
                .into_response();
            if let Ok(hdr) = HeaderValue::from_str(&retry_after_secs.to_string()) {
                res.headers_mut().insert(axum::http::header::RETRY_AFTER, hdr);
            }
            return res;
        }
        lumis_core::models::QuotaCheckResult::Allowed { .. } => {}
    }

    // 3. Guardrail pre-flight inspection (Prompt Injection blocking & PII sanitization)
    let sanitized_content = match state.guardrail.inspect(&payload.content) {
        lumis_core::traits::GuardrailVerdict::Blocked { reason, rule_matched } => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Security guardrail violation: rule '{}' triggered: {}", rule_matched, reason),
                }),
            )
                .into_response();
        }
        lumis_core::traits::GuardrailVerdict::Pass { sanitized_text } => sanitized_text,
    };

    // 4. Append User Message
    let user_msg = NewMessage {
        session_id: session_id.clone(),
        role: MessageRole::User,
        content: sanitized_content.clone(),
        model_id: payload.model_id.clone(),
        token_count: Some(estimated_tokens as u32),
        metadata: None,
    };

    if let Err(e) = state.repo.append_message(&user_msg).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response();
    }

    // 5. Record FinOps token usage & OpenTelemetry telemetry event
    let usage_record = lumis_core::models::TokenUsageRecord {
        tenant_id: tenant_id.to_string(),
        user_id: username,
        role,
        model_id: payload.model_id.clone().unwrap_or_else(|| "default-model".to_string()),
        provider: "openai".to_string(),
        input_tokens: estimated_tokens,
        output_tokens: 0,
        duration_ms: 10,
        timestamp: Utc::now(),
    };
    let _ = state.quota_engine.record_usage(&usage_record).await;
    state.telemetry.record_gen_ai_event(&usage_record);

    // 6. Broadcast to zero-copy streaming channel
    let event_payload = serde_json::json!({
        "event": "user_message",
        "session_id": session_id,
        "content": sanitized_content.clone(),
    });
    let _ = state
        .stream_tx
        .send(bytes::Bytes::from(event_payload.to_string()));

    // 7. If an LLM inference router is active, trigger streaming completion in background
    if let Some(router) = state.llm_router.clone() {
        let stream_tx = state.stream_tx.clone();
        let repo = state.repo.clone();
        let sess_id = session_id.clone();
        let model = payload.model_id.clone().unwrap_or_else(|| "gpt-4o-mini".to_string());
        let prompt = sanitized_content;

        tokio::spawn(async move {
            let req = lumis_core::models::ChatCompletionRequest {
                model: model.clone(),
                messages: vec![lumis_core::models::ChatMessagePayload {
                    role: "user".to_string(),
                    content: prompt,
                }],
                temperature: Some(0.7),
                max_tokens: None,
                stream: true,
            };

            if let Ok(mut rx) = router.complete_chat(&req).await {
                let mut accumulated = String::new();
                while let Some(chunk_res) = rx.recv().await {
                    if let Ok(chunk) = chunk_res {
                        accumulated.push_str(&chunk.content);
                        let chunk_event = serde_json::json!({
                            "event": "chunk",
                            "session_id": sess_id,
                            "delta": chunk.content,
                            "is_done": chunk.is_done,
                        });
                        let _ = stream_tx.send(bytes::Bytes::from(chunk_event.to_string()));
                        if chunk.is_done {
                            break;
                        }
                    }
                }

                if !accumulated.is_empty() {
                    let assistant_msg = NewMessage {
                        session_id: sess_id,
                        role: MessageRole::Assistant,
                        content: accumulated,
                        model_id: Some(model),
                        token_count: None,
                        metadata: None,
                    };
                    let _ = repo.append_message(&assistant_msg).await;
                }
            }
        });
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "received",
            "session_id": session_id,
        })),
    )
        .into_response()
}
