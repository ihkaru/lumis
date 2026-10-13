use chrono::Utc;
use lumis_core::models::{ChatMessagePayload, Message, MessageRole, Session, User, UserMemory};

pub const DEFAULT_MAX_CONTEXT_TOKENS: usize = 8192;

/// Assemble the complete chat completion messages array for an inference turn.
/// Enforces:
/// 1. Stable prefix system prompt (base + session custom + persistent user memories)
/// 2. Sliding window history pruning with token budget enforcement
/// 3. Tail dynamic context injection (temporal/spatial stamp cache-friendly)
/// 4. Current user prompt
/// Map internal model identifier to readable identity name
pub fn format_model_identity(model_id: &str) -> &'static str {
    match model_id {
        "cbai/glm-5.2" => "GLM 5.2 (Zhipu AI Flagship)",
        "cbai/glm-5v-turbo" => "GLM 5V Turbo (Multimodal Vision)",
        "cbai/minimax-m3" => "MiniMax M3 (1M Long-Context)",
        "cbai/kimi-k2.6" => "Kimi K2.6 (Moonshot AI)",
        "cbai/kimi-k2.5" => "Kimi K2.5 (High Speed)",
        "openrouter-free" => "Ling 3.1 Flash (OpenRouter)",
        "my9model-free" | "my9model-smart" | "my9model-fast" => "NVIDIA Nemotron 3 Ultra (9Router Gateway)",
        _ => "Enterprise AI Gateway",
    }
}

/// Assemble the complete chat completion messages array for an inference turn.
/// Enforces:
/// 1. Stable prefix system prompt (base + session custom + persistent user memories + active engine)
/// 2. Sliding window history pruning with token budget enforcement & duplicate prompt elimination
/// 3. Tail dynamic context injection (temporal/spatial stamp cache-friendly)
/// 4. Current user prompt
pub fn assemble_chat_context(
    session: &Session,
    user: &User,
    history: &[Message],
    current_query: &str,
    active_model: &str,
    user_memories: &[UserMemory],
    max_context_tokens: Option<usize>,
) -> Vec<ChatMessagePayload> {
    let budget = max_context_tokens.unwrap_or(DEFAULT_MAX_CONTEXT_TOKENS);
    let mut messages = Vec::new();

    // 1. Build Base System Prompt + Model Identity + Custom Session Prompt + User Memories
    let model_display = format_model_identity(active_model);
    let mut system_text = format!(
        "You are Lumis, an enterprise-grade AI assistant. Your current active inference engine is {}.\nProvide helpful, accurate, well-formatted markdown responses. Use LaTeX ($...$ or $$...$$) for formulas and mermaid for diagrams when helpful.\nIf asked about your identity or underlying model, accurately explain that you are Lumis powered by {}.",
        model_display, model_display
    );

    if let Some(ref custom) = session.system_prompt {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            system_text.push_str("\n\nSession Instructions:\n");
            system_text.push_str(trimmed);
        }
    }

    if !user_memories.is_empty() {
        system_text.push_str("\n\n<user_memories>\n");
        for mem in user_memories {
            system_text.push_str(&format!("- [{}] {}\n", mem.category, mem.content.trim()));
        }
        system_text.push_str("</user_memories>");
    }

    let system_tokens = (system_text.len() / 4).max(1);
    let current_tokens = (current_query.len() / 4).max(1);
    let reserved_tail = 128; // for temporal context stamp
    let history_budget = budget.saturating_sub(system_tokens + current_tokens + reserved_tail);

    messages.push(ChatMessagePayload {
        role: "system".to_string(),
        content: system_text,
    });

    // 2. Sliding-Window Budget Pruning on History
    // Exclude the current query if it was already appended to history
    let effective_history = if let Some(last) = history.last() {
        if last.role == MessageRole::User && last.content == current_query {
            &history[..history.len() - 1]
        } else {
            history
        }
    } else {
        history
    };

    let mut selected_history = Vec::new();
    let mut accumulated_tokens = 0;
    let mut pruned_any = false;

    // Traverse from newest to oldest
    for msg in effective_history.iter().rev() {
        let role_str = match msg.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
            MessageRole::Tool => "tool",
        };
        let msg_tokens = (msg.content.len() / 4).max(1);
        if accumulated_tokens + msg_tokens <= history_budget {
            accumulated_tokens += msg_tokens;
            selected_history.push(ChatMessagePayload {
                role: role_str.to_string(),
                content: msg.content.clone(),
            });
        } else {
            pruned_any = true;
        }
    }

    selected_history.reverse();

    if pruned_any {
        messages.push(ChatMessagePayload {
            role: "system".to_string(),
            content: "[Context note: Earlier conversation turns have been pruned to fit context window.]".to_string(),
        });
    }

    messages.extend(selected_history);

    // 3. Tail Dynamic Context Stamp (Temporal / Regional Awareness)
    let now = Utc::now();
    let timestamp_str = now.to_rfc3339();
    let tail_context = format!(
        "[System Context: Current Time: {} | User: {} | Session: '{}']",
        timestamp_str, user.username, session.title
    );
    messages.push(ChatMessagePayload {
        role: "system".to_string(),
        content: tail_context,
    });

    // 4. Current User Prompt
    messages.push(ChatMessagePayload {
        role: "user".to_string(),
        content: current_query.to_string(),
    });

    messages
}

/// Extract a concise, clean title (3-5 words, max 36 chars) from the first user query.
pub fn extract_heuristic_title(query: &str) -> String {
    let clean = query
        .replace(['\n', '\r', '#', '*', '`', '"', '\''], " ")
        .trim()
        .to_string();

    let words: Vec<&str> = clean.split_whitespace().collect();
    if words.is_empty() {
        return "Percakapan Baru".to_string();
    }

    let mut title = String::new();
    for word in words.iter().take(5) {
        if !title.is_empty() {
            title.push(' ');
        }
        title.push_str(word);
        if title.len() >= 32 {
            break;
        }
    }

    if title.len() > 36 {
        title.truncate(36);
        title = title.trim_end().to_string();
    }

    title
}
