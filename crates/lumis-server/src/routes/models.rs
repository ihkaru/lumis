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
    let (_username, role) = resolve_claims_and_role(&state, &headers);

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
    vec![
        ModelDescriptor {
            id: "cbai/glm-5.2".to_string(),
            name: "GLM 5.2 (Flagship Reasoning)".to_string(),
            provider: "CodeBuddy AI".to_string(),
            owned_by: Some("cbai".to_string()),
            tier: "Reasoning".to_string(),
            capabilities: Some(ModelCapabilities {
                vision: false,
                tools: true,
                reasoning: true,
                context_length: Some(200000),
                max_output: Some(128000),
            }),
            is_default: default_id == "cbai/glm-5.2",
            can_access: true,
        },
        ModelDescriptor {
            id: "cbai/glm-5v-turbo".to_string(),
            name: "GLM 5V Turbo (Vision)".to_string(),
            provider: "CodeBuddy AI".to_string(),
            owned_by: Some("cbai".to_string()),
            tier: "Advanced".to_string(),
            capabilities: Some(ModelCapabilities {
                vision: true,
                tools: true,
                reasoning: true,
                context_length: Some(200000),
                max_output: Some(128000),
            }),
            is_default: default_id == "cbai/glm-5v-turbo",
            can_access: true,
        },
        ModelDescriptor {
            id: "cbai/minimax-m3".to_string(),
            name: "MiniMax M3 (1M Long-Context)".to_string(),
            provider: "CodeBuddy AI".to_string(),
            owned_by: Some("cbai".to_string()),
            tier: "Advanced".to_string(),
            capabilities: Some(ModelCapabilities {
                vision: true,
                tools: true,
                reasoning: true,
                context_length: Some(1048576),
                max_output: Some(512000),
            }),
            is_default: default_id == "cbai/minimax-m3",
            can_access: true,
        },
        ModelDescriptor {
            id: "cbai/kimi-k2.6".to_string(),
            name: "Kimi K2.6 (Moonshot AI)".to_string(),
            provider: "CodeBuddy AI".to_string(),
            owned_by: Some("cbai".to_string()),
            tier: "Advanced".to_string(),
            capabilities: Some(ModelCapabilities {
                vision: true,
                tools: true,
                reasoning: true,
                context_length: Some(262144),
                max_output: Some(262144),
            }),
            is_default: default_id == "cbai/kimi-k2.6",
            can_access: true,
        },
        ModelDescriptor {
            id: "cbai/kimi-k2.5".to_string(),
            name: "Kimi K2.5 (High Speed)".to_string(),
            provider: "CodeBuddy AI".to_string(),
            owned_by: Some("cbai".to_string()),
            tier: "Advanced".to_string(),
            capabilities: Some(ModelCapabilities {
                vision: true,
                tools: true,
                reasoning: true,
                context_length: Some(262144),
                max_output: Some(262144),
            }),
            is_default: default_id == "cbai/kimi-k2.5",
            can_access: true,
        },
        ModelDescriptor {
            id: "openrouter-free".to_string(),
            name: "Ling 3.1 Flash (OpenRouter)".to_string(),
            provider: "OpenRouter Gateway".to_string(),
            owned_by: Some("openrouter".to_string()),
            tier: "Basic".to_string(),
            capabilities: Some(ModelCapabilities {
                vision: false,
                tools: true,
                reasoning: true,
                context_length: Some(131072),
                max_output: Some(8192),
            }),
            is_default: default_id == "openrouter-free",
            can_access: true,
        },
        ModelDescriptor {
            id: "my9model-free".to_string(),
            name: "Nemotron 3 Ultra (Free Gateway)".to_string(),
            provider: "9Router Gateway".to_string(),
            owned_by: Some("combo".to_string()),
            tier: "Basic".to_string(),
            capabilities: Some(ModelCapabilities {
                vision: false,
                tools: true,
                reasoning: true,
                context_length: Some(131072),
                max_output: Some(8192),
            }),
            is_default: default_id == "my9model-free",
            can_access: true,
        },
    ]
}
