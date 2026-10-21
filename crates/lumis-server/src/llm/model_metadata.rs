pub fn is_verified_model(id: &str) -> bool {
    let trimmed = id.trim();
    if trimmed.is_empty() || trimmed.contains("cooldown") {
        return false;
    }
    if let Ok(allowed) = std::env::var("ALLOWED_MODELS") {
        let list: Vec<&str> = allowed.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if !list.is_empty() {
            return list.contains(&trimmed);
        }
    }
    true
}

pub fn format_model_display_name(id: &str) -> String {
    let trimmed = id.trim();
    match trimmed {
        "my9model-free" => "Nemotron 3 Ultra (Free Gateway)".to_string(),
        "openrouter-free" => "Ling 3.1 Flash (OpenRouter)".to_string(),
        "my9model-smart" => "Nemotron 3 Smart".to_string(),
        "my9model-fast" => "Nemotron 3 Fast".to_string(),
        "cbai/glm-5.2" => "GLM 5.2 (Flagship Reasoning)".to_string(),
        "cbai/glm-5v-turbo" => "GLM 5V Turbo (Vision)".to_string(),
        "cbai/minimax-m3" => "MiniMax M3 (1M Long-Context)".to_string(),
        "cbai/kimi-k2.6" => "Kimi K2.6 (Moonshot AI)".to_string(),
        "cbai/kimi-k2.5" => "Kimi K2.5 (High Speed)".to_string(),
        "muse/muse-spark-1.3-contributor" => "Muse Spark 1.3 Contributor (Deep Reasoning)".to_string(),
        "muse/muse-spark-1.3" => "Muse Spark 1.3 (Balanced)".to_string(),
        "muse/muse-wonder-1.0" => "Muse Wonder 1.0 (Flagship)".to_string(),
        "muse/sam-3.1" => "SAM 3.1 (Meta Vision)".to_string(),
        other => dynamic_format_model_title(other),
    }
}

fn dynamic_format_model_title(raw_id: &str) -> String {
    let (provider, model_part) = match raw_id.split_once('/') {
        Some((p, m)) => (Some(p), m),
        None => (None, raw_id),
    };

    let clean_model = if let Some(p) = provider {
        let prefix_dash = format!("{}-", p);
        model_part.strip_prefix(&prefix_dash).unwrap_or(model_part)
    } else {
        model_part
    };

    let words: Vec<String> = clean_model
        .split(|c| c == '-' || c == '_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect();

    let title = words.join(" ");
    if let Some(p) = provider {
        let provider_title = match p.to_lowercase().as_str() {
            "cbai" => "CodeBuddy".to_string(),
            "muse" => "Muse".to_string(),
            other => {
                let mut c = other.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => other.to_string(),
                }
            }
        };
        if !title.to_lowercase().starts_with(&provider_title.to_lowercase()) {
            format!("{} {}", provider_title, title)
        } else {
            title
        }
    } else {
        title
    }
}

pub fn format_model_provider(id: &str, owned_by: Option<&str>) -> String {
    if let Some(owner) = owned_by.filter(|o| !o.trim().is_empty()) {
        match owner.to_lowercase().as_str() {
            "combo" => "9Router Gateway".to_string(),
            "muse" => "Meta Muse AI".to_string(),
            "cbai" => "CodeBuddy AI".to_string(),
            "openai" => "OpenAI".to_string(),
            "anthropic" => "Anthropic".to_string(),
            "openrouter" => "OpenRouter".to_string(),
            other => {
                let mut c = other.chars();
                match c.next() {
                    Some(f) => format!("{} AI", f.to_uppercase().collect::<String>() + c.as_str()),
                    None => "AI Gateway".to_string(),
                }
            }
        }
    } else if let Some((prefix, _)) = id.split_once('/') {
        match prefix.to_lowercase().as_str() {
            "cbai" => "CodeBuddy AI".to_string(),
            "muse" => "Meta Muse AI".to_string(),
            other => {
                let mut c = other.chars();
                match c.next() {
                    Some(f) => format!("{} AI", f.to_uppercase().collect::<String>() + c.as_str()),
                    None => "AI Gateway".to_string(),
                }
            }
        }
    } else if id.starts_with("my9model") {
        "9Router Gateway".to_string()
    } else {
        "OpenAI Compatible".to_string()
    }
}

pub fn determine_model_tier(id: &str, is_reasoning: bool, is_vision: bool) -> String {
    let lower = id.to_lowercase();
    if is_reasoning
        || lower.contains("reasoning")
        || lower.contains("think")
        || lower.contains("thought")
        || lower.contains("r1")
        || lower.contains("o1")
        || lower.contains("o3")
        || lower.contains("contributor")
        || lower.contains("wonder")
        || lower.contains("qwq")
    {
        "Reasoning".to_string()
    } else if is_vision
        || lower.contains("vision")
        || lower.contains("image")
        || lower.contains("visual")
        || lower.contains("sam")
        || lower.contains("vl")
    {
        "Advanced".to_string()
    } else if lower.contains("free") || lower.contains("basic") {
        "Basic".to_string()
    } else {
        "Standard".to_string()
    }
}
