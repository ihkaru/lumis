pub fn is_verified_model(id: &str) -> bool {
    matches!(
        id,
        "cbai/glm-5.2"
            | "cbai/glm-5v-turbo"
            | "cbai/minimax-m3"
            | "cbai/kimi-k2.6"
            | "cbai/kimi-k2.5"
            | "openrouter-free"
            | "my9model-free"
    )
}

pub fn format_model_display_name(id: &str) -> String {
    match id {
        "my9model-free" => "Nemotron 3 Ultra (Free Gateway)".to_string(),
        "openrouter-free" => "Ling 3.1 Flash (OpenRouter)".to_string(),
        "cbai/glm-5.2" => "GLM 5.2 (Flagship Reasoning)".to_string(),
        "cbai/glm-5v-turbo" => "GLM 5V Turbo (Vision)".to_string(),
        "cbai/minimax-m3" => "MiniMax M3 (1M Long-Context)".to_string(),
        "cbai/kimi-k2.6" => "Kimi K2.6 (Moonshot AI)".to_string(),
        "cbai/kimi-k2.5" => "Kimi K2.5 (High Speed)".to_string(),
        other => other.to_string(),
    }
}

pub fn format_model_provider(id: &str, owned_by: Option<&str>) -> String {
    if id.starts_with("cbai/") || owned_by == Some("cbai") {
        "CodeBuddy AI".to_string()
    } else if owned_by == Some("combo") || id.starts_with("my9model") {
        "9Router Gateway".to_string()
    } else if let Some(owner) = owned_by {
        owner.to_string()
    } else {
        "OpenAI Compatible".to_string()
    }
}

pub fn determine_model_tier(id: &str, is_reasoning: bool, is_vision: bool) -> String {
    if is_reasoning {
        "Reasoning".to_string()
    } else if is_vision {
        "Advanced".to_string()
    } else if id.contains("free") {
        "Basic".to_string()
    } else {
        "Standard".to_string()
    }
}
