use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::Json;
use lumis_core::models::{ModelCapabilities, ModelDescriptor};

use crate::routes::chat::helpers::resolve_claims_and_role;
use crate::state::AppState;

/// GET /api/models & GET /api/v1/models
/// Dynamically discovers and serves available inference models from configured gateway (e.g. 9Router).
/// Evaluates role clearance per model so the UI can accurately present access state.
pub async fn list_models(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_username, role) = resolve_claims_and_role(&state, &headers).await;

    let default_model = std::env::var("DEFAULT_MODEL").unwrap_or_else(|_| "my9model-free".to_string());

    let mut models: Vec<ModelDescriptor> = if let Some(ref router) = state.llm_router {
        match router.list_models().await {
            Ok(discovered) if !discovered.is_empty() => discovered,
            _ => fallback_models(&default_model),
        }
    } else {
        fallback_models(&default_model)
    };

    // Policy Enforcement: Annotate clearance per caller role
    for m in &mut models {
        let clearance_ok = state
            .claim_translator
            .evaluate_model_clearance(role, &m.id)
            .is_ok();
        m.can_access = clearance_ok;
    }

    Json(models)
}

fn fallback_models(default_id: &str) -> Vec<ModelDescriptor> {
    use crate::llm::model_metadata::{determine_model_tier, format_model_display_name, format_model_provider};
    let mut models = vec![
        ModelDescriptor {
            id: default_id.to_string(),
            name: format_model_display_name(default_id),
            provider: format_model_provider(default_id, None),
            owned_by: None,
            tier: determine_model_tier(default_id, false, false),
            capabilities: Some(ModelCapabilities {
                vision: false,
                tools: true,
                reasoning: false,
                context_length: Some(131072),
                max_output: Some(8192),
            }),
            is_default: true,
            can_access: true,
        },
    ];

    if default_id != "openrouter-free" {
        models.push(ModelDescriptor {
            id: "openrouter-free".to_string(),
            name: format_model_display_name("openrouter-free"),
            provider: format_model_provider("openrouter-free", Some("openrouter")),
            owned_by: Some("openrouter".to_string()),
            tier: "Basic".to_string(),
            capabilities: Some(ModelCapabilities {
                vision: false,
                tools: true,
                reasoning: false,
                context_length: Some(131072),
                max_output: Some(8192),
            }),
            is_default: false,
            can_access: true,
        });
    }

    models
}
