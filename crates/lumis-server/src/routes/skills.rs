use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::{
    is_valid_skill_name, CreateSkillInput, UpdateSkillInput,
};
use serde::Deserialize;
use serde_json::json;

use crate::routes::mcp_user_servers::extract_tenant_and_user;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SkillListParams {
    pub search: Option<String>,
    pub category: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SkillSearchParams {
    pub q: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ToggleSkillPayload {
    pub enabled: bool,
}

/// GET /api/skills - List or search skills with FTS5 BM25 ranking
pub async fn list_skills_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<SkillListParams>,
) -> impl IntoResponse {
    let (tenant_id, user_id, _) = extract_tenant_and_user(&headers, &state).await;
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = params.offset.unwrap_or(0);

    if let Some(ref search) = params.search {
        if !search.trim().is_empty() {
            match state.repo.search_skills_fts(&tenant_id, &user_id, search, limit).await {
                Ok(results) => return (StatusCode::OK, Json(json!({ "skills": results }))).into_response(),
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
            }
        }
    }

    match state.repo.list_skills(&tenant_id, &user_id, params.category.as_deref(), limit, offset).await {
        Ok(skills) => (StatusCode::OK, Json(json!({ "skills": skills }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// GET /api/skills/search - Fast autocomplete search for composer $popover (< 0.5ms)
pub async fn search_skills_autocomplete_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<SkillSearchParams>,
) -> impl IntoResponse {
    let (tenant_id, user_id, _) = extract_tenant_and_user(&headers, &state).await;
    let query_str = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(10).clamp(1, 30);

    if query_str.trim().is_empty() {
        // Return top default active skills
        match state.repo.list_skills(&tenant_id, &user_id, None, limit, 0).await {
            Ok(skills) => return (StatusCode::OK, Json(json!({ "skills": skills }))).into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
        }
    }

    match state.repo.search_skills_fts(&tenant_id, &user_id, &query_str, limit).await {
        Ok(results) => {
            let summaries: Vec<_> = results.into_iter().map(|r| r.skill).collect();
            (StatusCode::OK, Json(json!({ "skills": summaries }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// GET /api/skills/:id - Get full skill details
pub async fn get_skill_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (tenant_id, _, _) = extract_tenant_and_user(&headers, &state).await;
    match state.repo.get_skill_by_id(&tenant_id, &id).await {
        Ok(Some(skill)) => (StatusCode::OK, Json(json!({ "skill": skill }))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Skill not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// POST /api/skills - Create a new skill
pub async fn create_skill_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateSkillInput>,
) -> impl IntoResponse {
    let (tenant_id, user_id, _) = extract_tenant_and_user(&headers, &state).await;
    if !is_valid_skill_name(&input.name) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Skill name must be lowercase kebab-case (a-z0-9 and hyphens), 1-64 characters" })),
        )
            .into_response();
    }

    if input.description.trim().is_empty() || input.body.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Skill description and body are required" })),
        )
            .into_response();
    }

    match state.repo.create_skill(&tenant_id, &user_id, &input).await {
        Ok(skill) => (StatusCode::CREATED, Json(json!({ "skill": skill }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// PUT /api/skills/:id - Update skill with optimistic concurrency
pub async fn update_skill_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<UpdateSkillInput>,
) -> impl IntoResponse {
    let (tenant_id, _, _) = extract_tenant_and_user(&headers, &state).await;
    match state.repo.update_skill(&tenant_id, &id, &input).await {
        Ok(skill) => (StatusCode::OK, Json(json!({ "skill": skill }))).into_response(),
        Err(lumis_core::error::LumisError::Conflict(msg)) => {
            (StatusCode::CONFLICT, Json(json!({ "error": msg }))).into_response()
        }
        Err(lumis_core::error::LumisError::NotFound(msg)) => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": msg }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// DELETE /api/skills/:id - Soft-delete skill
pub async fn delete_skill_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (tenant_id, _, _) = extract_tenant_and_user(&headers, &state).await;
    match state.repo.delete_skill(&tenant_id, &id).await {
        Ok(true) => (StatusCode::OK, Json(json!({ "deleted": true }))).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Skill not found" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// POST /api/skills/:id/toggle - Toggle user skill activation
pub async fn toggle_skill_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<ToggleSkillPayload>,
) -> impl IntoResponse {
    let (_, user_id, _) = extract_tenant_and_user(&headers, &state).await;
    match state.repo.toggle_user_skill(&user_id, &id, payload.enabled).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true, "is_enabled": payload.enabled }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}
