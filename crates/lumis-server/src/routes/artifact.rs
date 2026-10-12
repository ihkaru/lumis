use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use lumis_core::models::{
    ArtifactIpcAction, ArtifactIpcRequest, ArtifactIpcResponse, ArtifactParser,
    ArtifactSpecification, AuditEventStatus, AuditLogEvent,
};
use serde::{Deserialize, Serialize};

use crate::routes::chat::helpers::resolve_claims_and_role;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ParseArtifactsRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct ParseArtifactsResponse {
    pub artifacts: Vec<ArtifactSpecification>,
    pub count: usize,
}

#[derive(Debug, Deserialize)]
pub struct ArtifactIpcEnvelope {
    pub artifact_id: String,
    pub session_id: Option<String>,
    pub request: ArtifactIpcRequest,
}

/// Parses XML/fenced artifact containers from model output.
pub async fn parse_artifacts(
    Json(payload): Json<ParseArtifactsRequest>,
) -> impl IntoResponse {
    let artifacts = ArtifactParser::parse_artifacts(&payload.text);
    let count = artifacts.len();
    (StatusCode::OK, Json(ParseArtifactsResponse { artifacts, count })).into_response()
}

/// Secure PostMessage IPC Bridge endpoint.
/// Allows sandboxed iframes to request model completions & tool calls without exposing API keys.
pub async fn handle_artifact_ipc(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ArtifactIpcEnvelope>,
) -> impl IntoResponse {
    let (username, role) = resolve_claims_and_role(&state, &headers);
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default");

    // 1. SOC Killswitch Enforcement
    if state.audit_logger.is_killswitch_active() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ArtifactIpcResponse {
                request_id: payload.request.request_id,
                success: false,
                data: None,
                error: Some("Global Emergency Killswitch Active".to_string()),
            }),
        )
            .into_response();
    }

    let req_id = payload.request.request_id.clone();

    // 2. Process Actions
    let response = match payload.request.action {
        ArtifactIpcAction::Ready => ArtifactIpcResponse {
            request_id: req_id,
            success: true,
            data: Some(serde_json::json!({
                "status": "connected",
                "artifact_id": payload.artifact_id,
                "role": role.as_str(),
            })),
            error: None,
        },
        ArtifactIpcAction::ModelPrompt { prompt } => {
            // Guardrail inspection
            match state.guardrail.inspect(&prompt) {
                lumis_core::traits::GuardrailVerdict::Blocked { reason, rule_matched } => {
                    ArtifactIpcResponse {
                        request_id: req_id,
                        success: false,
                        data: None,
                        error: Some(format!("Guardrail blocked: {} ({})", rule_matched, reason)),
                    }
                }
                lumis_core::traits::GuardrailVerdict::Pass { sanitized_text } => {
                    let reply_msg = format!("Completed prompt: {}", sanitized_text);
                    ArtifactIpcResponse {
                        request_id: req_id,
                        success: true,
                        data: Some(serde_json::json!({
                            "response": reply_msg,
                            "sanitized": sanitized_text != prompt,
                        })),
                        error: None,
                    }
                }
            }
        }
        ArtifactIpcAction::ToolInvoke { tool, args } => {
            // Check if tool exists in registry
            let tools = state.mcp_registry.list_tools(role).await;
            if let Some(target_tool) = tools.iter().find(|t| t.name == tool) {
                ArtifactIpcResponse {
                    request_id: req_id,
                    success: true,
                    data: Some(serde_json::json!({
                        "tool": target_tool.name,
                        "description": target_tool.description,
                        "result": "executed",
                        "args": args,
                    })),
                    error: None,
                }
            } else {
                ArtifactIpcResponse {
                    request_id: req_id,
                    success: false,
                    data: None,
                    error: Some(format!("Tool '{}' not registered in MCP registry", tool)),
                }
            }
        }
    };

    // 3. SIEM Audit Logging
    let _ = state
        .audit_logger
        .log_event(&AuditLogEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            tenant_id: tenant_id.to_string(),
            actor_id: username,
            actor_role: role,
            action: format!("artifact_ipc:{}", payload.artifact_id),
            resource_id: Some(payload.artifact_id),
            status: if response.success {
                AuditEventStatus::Success
            } else {
                AuditEventStatus::Failure
            },
            ip_address: None,
            user_agent: None,
            details: serde_json::json!({ "request_id": response.request_id }),
        })
        .await;

    (StatusCode::OK, Json(response)).into_response()
}
