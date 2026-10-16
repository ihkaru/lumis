use chrono::Utc;
use lumis_core::models::{
    ChatMessagePayload, McpToolDefinition, Message, MessageRole, Session, Skill, SkillSummary, User, UserMemory,
};
use super::executor_helpers::extract_assistant_tool_turns;

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

pub use super::system_prompts::format_model_identity;
use super::system_prompts::build_base_system_prompt;

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
    let now = Utc::now();
    let mut system_text = build_base_system_prompt(active_model, now);

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
            system_text.push_str("You have access to the following on-demand enterprise skills in Lumis:\n");
            for s in skills {
                let desc = if s.description.len() > 250 {
                    format!("{}...", &s.description[..247])
                } else {
                    s.description.clone()
                };
                system_text.push_str(&format!("- `${}` ({}): {}\n", s.name, s.display_title, desc));
            }
            system_text.push_str("Apply relevant skill directives when the user's task matches the skill description or when the user invokes `$skill-name`.\n</available_skills>\n");
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
        if msg.role == MessageRole::Assistant {
            if let Some((tool_calls, tool_results)) = extract_assistant_tool_turns(msg) {
                let pre_content = msg.metadata.get("pre_content")
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .unwrap_or("");

                let mut turn_msgs = Vec::new();

                // 1. Assistant message that called the tools
                turn_msgs.push(ChatMessagePayload {
                    role: "assistant".to_string(),
                    content: pre_content.to_string(),
                    tool_calls: Some(tool_calls),
                    tool_call_id: None,
                });

                // 2. Respective Tool messages
                for (cid, res) in tool_results {
                    let clean_res = if res.len() > 3000 {
                        super::executor_helpers::compress_observation(&res, 2000)
                    } else {
                        res
                    };
                    turn_msgs.push(ChatMessagePayload {
                        role: "tool".to_string(),
                        content: clean_res,
                        tool_calls: None,
                        tool_call_id: Some(cid),
                    });
                }

                // 3. Final assistant text (if any and not just repeating pre_content)
                let trimmed_content = msg.content.trim();
                if !trimmed_content.is_empty() && trimmed_content != pre_content.trim() {
                    turn_msgs.push(ChatMessagePayload {
                        role: "assistant".to_string(),
                        content: trimmed_content.to_string(),
                        tool_calls: None,
                        tool_call_id: None,
                    });
                }

                let total_tokens: usize = turn_msgs.iter().map(|m| (m.content.len() / 4).max(1)).sum();
                if accumulated_tokens + total_tokens <= history_budget {
                    accumulated_tokens += total_tokens;
                    for tm in turn_msgs.into_iter().rev() {
                        selected_history.push(tm);
                    }
                } else {
                    pruned_any = true;
                }
                continue;
            }
        }

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
