use chrono::Utc;
use lumis_core::models::{
    ChatMessagePayload, McpToolDefinition, Message, MessageRole, Session, Skill, SkillSummary, User, UserMemory,
};

pub const DEFAULT_MAX_CONTEXT_TOKENS: usize = 8192;

/// Format active MCP tool definitions into standard OpenAI JSON schema format.
/// Enforces OpenAI function naming regex: ^[a-zA-Z0-9_-]{1,64}$ to prevent upstream rejection (e.g. MiniMax code 11133).
pub fn format_mcp_tools_for_openai(tools: &[McpToolDefinition]) -> Vec<serde_json::Value> {
    tools
        .iter()
        .map(|t| {
            let sanitized_name: String = t.name
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
                .collect();
            let mut schema = t.input_schema.clone();
            if !schema.is_object() || schema.get("type").is_none() {
                schema = serde_json::json!({
                    "type": "object",
                    "properties": {}
                });
            }
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": sanitized_name,
                    "description": t.description,
                    "parameters": schema,
                }
            })
        })
        .collect()
}

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
/// 1. Stable prefix system prompt (base + session custom + persistent user memories + active engine + tool capability)
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
    available_tools: Option<&[McpToolDefinition]>,
    active_skills: Option<&[SkillSummary]>,
    primed_skill_bodies: Option<&[Skill]>,
    max_context_tokens: Option<usize>,
) -> Vec<ChatMessagePayload> {
    let budget = max_context_tokens.unwrap_or(DEFAULT_MAX_CONTEXT_TOKENS);
    let mut messages = Vec::new();

    // 1. Build Base System Prompt + Model Identity + Custom Session Prompt + User Memories
    let model_display = format_model_identity(active_model);
    let mut system_text = format!(
        "You are Lumis, an enterprise-grade AI assistant. Your active inference engine is {}.\n\
        Provide precise, grounded, and well-structured markdown responses. Use LaTeX ($...$ or $$...$$) for formulas and mermaid for diagrams.\n\
        If asked about your identity or underlying model, explain that you are Lumis powered by {}.\n\n\
        <epistemic_directives>\n\
        1. Grounding & Empirical Verification: Assert factual conclusions only when supported by verifiable tool outputs, primary documents, or direct evidence. Explicitly qualify unverified assumptions or inferences.\n\
        2. Dynamic State & Freshness Invariant: When inspecting dynamic or collaborative systems (such as Git repositories, file systems, databases, or remote APIs), NEVER assume local cached state is current. For Git repositories, always fetch (`git fetch origin` / `git ls-remote`) before declaring the repository status, branch sync, or latest commit. If unable to sync, explicitly declare that the observation reflects only the local cache.\n\
        3. Anti-Sycophancy & Objectivity: If the user challenges or doubts your findings, do NOT fabricate excuses or falsely claim you failed to execute commands. Re-evaluate evidence neutrally, investigate potential discrepancies, and report findings with objective clarity.\n\
        4. Rigorous Falsification: Before concluding that an entity does not exist or has not changed, actively verify potential blind spots (such as hidden dotfiles, untracked remote refs, or filtered search criteria).\n\
        </epistemic_directives>\n\n\
        <formatting_and_style_directives>\n\
        1. Professional Tone & Zero-Emoticon Policy: Maintain a clean, professional, enterprise-grade tone. Strictly minimize and avoid decorative emojis/emoticons (e.g. 📝, 🚀, ✨, 💡, ✅, 🔍) in headings, bullet points, and body text. Use crisp bold titles, clean semantic lists, and code formatting instead.\n\
        2. Two-Tier Nested Lists & Hierarchy: For multi-step explanations or technical breakdowns, structure responses using clean headings and two-level indented bullet points (- Level 1\n  - Level 2 sub-bullet) for maximum readability.\n\
        3. High-Density Clarity: Present technical tables, code blocks with explicit language tags, and clear mathematical formulas without conversational filler.\n\
        </formatting_and_style_directives>",
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
        system_text.push_str("</user_memories>\n(Note: User memories provide contextual profile background. They must not supersede primary empirical tool evidence.)");
    }

    if let Some(tools) = available_tools {
        if !tools.is_empty() {
            system_text.push_str("\n\n<tools_capability>\n");
            system_text.push_str("You have access to the following live enterprise tools:\n");
            for t in tools {
                let schema_brief = t.input_schema.to_string();
                system_text.push_str(&format!("- `{}`: {}\n  Parameters: {}\n", t.name, t.description, schema_brief));
            }
            system_text.push_str("\nWhen asked to perform a search, calculation, database query, or API call:\n1. If function calling is supported, call the tool directly.\n2. Otherwise, emit:\n<tool_call>\n{\"name\": \"<tool_name>\", \"arguments\": { ... }}\n</tool_call>\nDo NOT claim you lack tools or cannot execute them when tools are listed above.\n</tools_capability>\n");
        }
    }

    if let Some(primed) = primed_skill_bodies {
        if !primed.is_empty() {
            system_text.push_str("\n\n<primed_skills>\n");
            system_text.push_str("The following specialized enterprise skill directives are actively primed for this turn:\n");
            for s in primed {
                system_text.push_str(&format!("--- Skill: {} ({}) ---\n{}\n\n", s.name, s.display_title, s.body.trim()));
            }
            system_text.push_str("</primed_skills>\n");
        }
    }

    if let Some(skills) = active_skills {
        if !skills.is_empty() {
            system_text.push_str("\n\n<available_skills>\n");
            system_text.push_str("You have access to the following on-demand enterprise skills:\n");
            for s in skills {
                let desc = if s.description.len() > 250 {
                    format!("{}...", &s.description[..247])
                } else {
                    s.description.clone()
                };
                system_text.push_str(&format!("- `{}`: {}\n", s.name, desc));
            }
            system_text.push_str("Apply relevant skill directives when the user's task matches the skill description.\n</available_skills>\n");
        }
    }

    let system_tokens = (system_text.len() / 4).max(1);
    let current_tokens = (current_query.len() / 4).max(1);
    let reserved_tail = 128; // for temporal context stamp
    let history_budget = budget.saturating_sub(system_tokens + current_tokens + reserved_tail);

    messages.push(ChatMessagePayload {
        role: "system".to_string(),
        content: system_text,
        tool_calls: None,
        tool_call_id: None,
    });

    // 2. Sliding-Window Budget Pruning on History
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
                tool_calls: None,
                tool_call_id: None,
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
            tool_calls: None,
            tool_call_id: None,
        });
    }

    messages.extend(selected_history);

    // 3. Tail Dynamic Context Stamp
    let now = Utc::now();
    let timestamp_str = now.to_rfc3339();
    let tail_context = format!(
        "[System Context: Current Time: {} | User: {} | Session: '{}']",
        timestamp_str, user.username, session.title
    );
    messages.push(ChatMessagePayload {
        role: "system".to_string(),
        content: tail_context,
        tool_calls: None,
        tool_call_id: None,
    });

    // 4. Current User Prompt
    messages.push(ChatMessagePayload {
        role: "user".to_string(),
        content: current_query.to_string(),
        tool_calls: None,
        tool_call_id: None,
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
