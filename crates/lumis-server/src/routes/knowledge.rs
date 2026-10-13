use std::collections::HashMap;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::{EnterpriseRole, KnowledgeQuery};
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct IngestDocumentRequest {
    pub title: String,
    pub content: String,
    pub content_type: Option<String>,
    pub min_role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryKnowledgeRequest {
    pub query: String,
    pub top_k: Option<usize>,
}

async fn resolve_tenant_and_role(state: &AppState, headers: &HeaderMap) -> (String, EnterpriseRole) {
    let mut header_map = HashMap::new();
    for (k, v) in headers.iter() {
        if let Ok(str_val) = v.to_str() {
            header_map.insert(k.as_str().to_lowercase(), str_val.to_string());
        }
    }
    if let Some(val) = headers.get(&state.trusted_header).and_then(|v| v.to_str().ok()) {
        header_map.insert("x-forwarded-user".to_string(), val.to_string());
    }

    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default")
        .to_string();

    let role = if let Ok(claims) = state.claim_translator.extract_claims(&header_map) {
        if !claims.roles.is_empty() {
            state.claim_translator.resolve_effective_role(&claims)
        } else {
            match state.repo.get_user_by_username(&claims.username).await {
                Ok(Some(u)) => u.role,
                _ => EnterpriseRole::Member,
            }
        }
    } else {
        EnterpriseRole::Member
    };

    (tenant_id, role)
}

pub async fn ingest_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<IngestDocumentRequest>,
) -> impl IntoResponse {
    let (tenant_id, caller_role) = resolve_tenant_and_role(&state, &headers).await;
    let min_role = payload
        .min_role
        .as_deref()
        .map(EnterpriseRole::from_str)
        .unwrap_or(EnterpriseRole::Member);

    // PEP Check: Caller cannot ingest a document with a clearance requirement higher than their own
    if !caller_role.is_at_least(min_role) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Cannot assign document clearance higher than caller's role level"
            })),
        )
            .into_response();
    }

    let content_type = payload.content_type.unwrap_or_else(|| "text/plain".to_string());
    match state
        .knowledge_vault
        .ingest_document(&tenant_id, &payload.title, &payload.content, &content_type, min_role)
        .await
    {
        Ok(doc) => (StatusCode::CREATED, Json(doc)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn list_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (tenant_id, caller_role) = resolve_tenant_and_role(&state, &headers).await;
    match state.knowledge_vault.list_documents(&tenant_id, caller_role).await {
        Ok(docs) => (StatusCode::OK, Json(docs)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn query_knowledge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<QueryKnowledgeRequest>,
) -> impl IntoResponse {
    let (tenant_id, caller_role) = resolve_tenant_and_role(&state, &headers).await;
    let query = KnowledgeQuery {
        query: payload.query,
        tenant_id,
        caller_role,
        top_k: payload.top_k.unwrap_or(5),
    };

    match state.knowledge_vault.query_knowledge(&query).await {
        Ok(results) => (StatusCode::OK, Json(results)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(document_id): Path<String>,
) -> impl IntoResponse {
    let (tenant_id, caller_role) = resolve_tenant_and_role(&state, &headers).await;
    match state
        .knowledge_vault
        .delete_document(&tenant_id, &document_id, caller_role)
        .await
    {
        Ok(true) => (StatusCode::NO_CONTENT).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Document not found" }))).into_response(),
        Err(e) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
