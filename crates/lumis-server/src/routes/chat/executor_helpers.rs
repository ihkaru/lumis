#![deny(unsafe_code)]

use std::sync::Arc;
use lumis_core::models::{
    ChatCompletionRequest, ChatMessageFunctionCall, ChatMessagePayload, ChatMessageToolCall,
    Message, Skill, SkillSearchResult, SkillSummary,
};
use lumis_core::traits::{ChatRepository, LlmProviderDriver};

/// Maximum agentic turns for long-running workflows.
/// Defaults to unbounded (usize::MAX) or configured via LUMIS_MAX_AGENT_TURNS (0 = infinite).
pub fn get_max_agentic_turns() -> usize {
    std::env::var("LUMIS_MAX_AGENT_TURNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .map(|n| if n == 0 { usize::MAX } else { n })
        .unwrap_or(usize::MAX)
}

/// Compresses historical tool observations exceeding character limits to prevent context drift and bloat.
pub fn compress_observation(text: &str, max_len: usize) -> String {
    if text.len() <= max_len { return text.to_string(); }
    let half = max_len / 2;
    let mut p_end = half;
    while p_end > 0 && !text.is_char_boundary(p_end) { p_end -= 1; }
    let mut s_start = text.len().saturating_sub(half);
    while s_start < text.len() && !text.is_char_boundary(s_start) { s_start += 1; }
    if p_end >= s_start { return text.to_string(); }
    format!("{}... [truncated {} bytes] ...{}", &text[..p_end], s_start.saturating_sub(p_end), &text[s_start..])
}

/// Heals or infers tool name if empty or omitted by upstream model using common argument signatures.
pub fn heal_or_infer_tool_name(raw_name: &str, args: &serde_json::Value) -> String {
    let trimmed = raw_name.trim();
    if !trimmed.is_empty() {
        return if trimmed == "get_skill" { "get_skill_directive".to_string() } else { trimmed.to_string() };
    }
    if args.get("code").is_some() { "execute_code".to_string() }
    else if args.get("query").is_some() {
        if args.get("category").is_some() || args.get("limit").is_some() { "search_skills".to_string() } else { "gateway_search_tools".to_string() }
    } else if args.get("goal").is_some() { "gateway_plan_tasks".to_string() }
    else if args.get("plan").is_some() { "gateway_execute_plan".to_string() }
    else if args.get("ticket_id").is_some() { "resolve_approval".to_string() }
    else if args.get("skill").is_some() || args.get("skill_name").is_some() { "get_skill_directive".to_string() }
    else { String::new() }
}

/// Parses pseudo-XML <tool_call> tags from models that output text rather than native function calling.
pub fn parse_text_tool_calls(text: &str) -> Vec<(String, String, serde_json::Value)> {
    let mut results = Vec::new();
    let mut cursor = text;
    while let Some(start_idx) = cursor.find("<tool_call>") {
        let after_start = &cursor[start_idx + 11..];
        if let Some(end_idx) = after_start.find("</tool_call>") {
            let json_str = after_start[..end_idx].trim();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                let name_candidate = val.get("name")
                    .or_else(|| val.get("tool"))
                    .or_else(|| val.get("function"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let args = if let Some(arguments) = val.get("arguments") {
                    arguments.clone()
                } else if let Some(params) = val.get("parameters") {
                    params.clone()
                } else {
                    let mut clean_obj = val.clone();
                    if let Some(map) = clean_obj.as_object_mut() {
                        map.remove("name");
                        map.remove("tool");
                        map.remove("function");
                    }
                    clean_obj
                };

                let resolved_name = heal_or_infer_tool_name(name_candidate, &args);
                if !resolved_name.is_empty() {
                    let call_id = format!("call_{}", uuid::Uuid::new_v4().simple());
                    results.push((call_id, resolved_name, args));
                }
            }
            cursor = &after_start[end_idx + 12..];
        } else {
            break;
        }
    }
    results
}

/// Evaluates if all planned session tasks have reached terminal completion (Convergence Gate).
/// Returns true if tasks exist and ALL are Completed or Cancelled.
pub async fn check_plan_convergence(repo: &Arc<dyn ChatRepository>, session_id: &str) -> bool {
    if let Ok(tasks) = repo.list_session_tasks(session_id).await {
        if !tasks.is_empty() {
            let all_terminal = tasks.iter().all(|t| t.status.is_terminal());
            return all_terminal;
        }
    }
    false
}

/// Extracts manually specified skill names ($skill-name) from user prompt.
pub fn extract_manual_skill_names(prompt: &str) -> Vec<String> {
    let mut names = Vec::new();
    for word in prompt.split_whitespace() {
        if let Some(stripped) = word.strip_prefix('$') {
            let clean: String = stripped
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            if !clean.is_empty() && !names.contains(&clean) {
                names.push(clean);
            }
        }
    }
    names
}

/// Asynchronously generates a concise, high-relevance title (3-5 words) from the first turn
/// using the active LLM driver, preventing raw heuristic query dumps.
pub async fn generate_llm_title(
    router: &Arc<dyn LlmProviderDriver>,
    model: &str,
    prompt: &str,
    assistant_text: &str,
) -> Option<String> {
    let clean_prompt = prompt.trim();
    if clean_prompt.is_empty() {
        return None;
    }

    let trunc_prompt = if clean_prompt.len() > 300 {
        &clean_prompt[..300]
    } else {
        clean_prompt
    };

    let clean_reply = assistant_text
        .replace("<tool_call>", "")
        .replace("</tool_call>", "")
        .trim()
        .to_string();

    let trunc_reply = if clean_reply.len() > 300 {
        &clean_reply[..300]
    } else {
        &clean_reply
    };

    let title_messages = vec![
        ChatMessagePayload::new(
            "system",
            "You are a conversation title generator. Output ONLY a concise, high-relevance title (3 to 5 words) that summarizes the conversation topic. Do NOT use quotes, punctuation, or emoji. Respond in the same language used by the user.",
        ),
        ChatMessagePayload::new(
            "user",
            format!("User: {}\nAssistant: {}", trunc_prompt, trunc_reply),
        ),
    ];

    let req = ChatCompletionRequest {
        model: model.to_string(),
        messages: title_messages,
        temperature: Some(0.3),
        max_tokens: Some(25),
        stream: true,
        tools: None,
        tool_choice: None,
    };

    let mut rx = match tokio::time::timeout(tokio::time::Duration::from_secs(4), router.complete_chat(&req)).await {
        Ok(Ok(receiver)) => receiver,
        _ => return None,
    };

    let mut title_buf = String::new();
    while let Ok(Some(chunk_res)) = tokio::time::timeout(tokio::time::Duration::from_millis(1500), rx.recv()).await {
        if let Ok(chunk) = chunk_res {
            title_buf.push_str(&chunk.content);
            if chunk.is_done || title_buf.len() >= 60 {
                break;
            }
        } else {
            break;
        }
    }

    let raw = title_buf
        .replace(['\n', '\r', '"', '\'', '`', '*', '#', '<', '>', '.', ':', ';'], " ")
        .trim()
        .to_string();

    let clean = if let Some(stripped) = raw.strip_prefix("Title").or_else(|| raw.strip_prefix("Judul")) {
        stripped.trim()
    } else {
        raw.as_str()
    };

    if clean.is_empty() {
        return None;
    }

    let words: Vec<&str> = clean.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    let mut final_title = words.into_iter().take(6).collect::<Vec<_>>().join(" ");
    if final_title.len() > 40 {
        final_title.truncate(40);
        final_title = final_title.trim_end().to_string();
    }

    Some(final_title)
}

/// Extracts tool calls and their results from an assistant message's metadata
/// to support full OpenAI-standard tool replay in multi-turn conversations.
pub fn extract_assistant_tool_turns(
    msg: &Message,
) -> Option<(Vec<ChatMessageToolCall>, Vec<(String, String)>)> {
    let meta = &msg.metadata;
    let tc_array = meta.get("tool_calls").and_then(|v: &serde_json::Value| v.as_array())
        .or_else(|| meta.get("parts").and_then(|v: &serde_json::Value| v.as_array()))?;
    if tc_array.is_empty() {
        return None;
    }

    let mut tool_calls = Vec::new();
    let mut tool_results = Vec::new();

    for (idx, tc) in tc_array.iter().enumerate() {
        if let Some(ptype) = tc.get("type").and_then(|t: &serde_json::Value| t.as_str()) {
            if ptype != "tool_call" {
                continue;
            }
        }

        let tool_name = tc.get("tool")
            .or_else(|| tc.get("name"))
            .and_then(|v: &serde_json::Value| v.as_str())
            .unwrap_or("")
            .to_string();
        if tool_name.is_empty() {
            continue;
        }

        let call_id = tc.get("call_id")
            .or_else(|| tc.get("id"))
            .and_then(|v: &serde_json::Value| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("call_{}_{}", msg.id, idx));

        let args = tc.get("args")
            .or_else(|| tc.get("arguments"))
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let args_str = if args.is_string() {
            args.as_str().unwrap().to_string()
        } else {
            args.to_string()
        };

        let result_str = tc.get("result")
            .or_else(|| tc.get("output"))
            .and_then(|v: &serde_json::Value| v.as_str())
            .unwrap_or("")
            .to_string();

        tool_calls.push(ChatMessageToolCall {
            id: call_id.clone(),
            r#type: "function".to_string(),
            function: ChatMessageFunctionCall {
                name: tool_name,
                arguments: args_str,
            },
        });

        tool_results.push((call_id, result_str));
    }

    if tool_calls.is_empty() {
        None
    } else {
        Some((tool_calls, tool_results))
    }
}

/// Resolves active and primed skills for context assembly.
pub async fn resolve_execution_skills(
    repo: &Arc<dyn ChatRepository>,
    tenant_id: &str,
    user_id: &str,
    prompt: &str,
) -> (Vec<Skill>, Vec<SkillSummary>, Vec<String>) {
    let mut primed = repo.get_always_apply_skills(tenant_id, user_id).await.unwrap_or_default();
    let manual_names = extract_manual_skill_names(prompt);
    for name in &manual_names {
        if let Ok(Some(s)) = repo.get_skill_by_name(tenant_id, name).await {
            if !primed.iter().any(|ps| ps.id == s.id) { primed.push(s); }
        }
    }
    let mut fts = repo.search_skills_fts(tenant_id, user_id, prompt, 5).await.unwrap_or_default();
    if fts.is_empty() {
        if let Ok(gen) = repo.list_skills(tenant_id, user_id, None, 5, 0).await {
            fts = gen.into_iter().map(|s| SkillSearchResult { skill: s, bm25_score: 0.0 }).collect();
        }
    }
    let summaries: Vec<_> = fts.into_iter().map(|r| r.skill).collect();
    (primed, summaries, manual_names)
}

/// Builds assistant metadata JSON payload safely.
pub fn build_assistant_metadata(
    executed_tools: &[serde_json::Value],
    message_parts: &[serde_json::Value],
    manual_skills: &[String],
    primed_skills: &[Skill],
) -> Option<serde_json::Value> {
    if executed_tools.is_empty() && message_parts.is_empty() { return None; }
    let mut m = serde_json::json!({ "tool_calls": executed_tools, "parts": message_parts });
    if let Some(txt) = message_parts.first().and_then(|p| p.get("text")).and_then(|t| t.as_str()) {
        m["pre_content"] = serde_json::json!(txt);
    }
    if !manual_skills.is_empty() { m["manual_skills"] = serde_json::json!(manual_skills); }
    if !primed_skills.is_empty() {
        let names: Vec<_> = primed_skills.iter().map(|s| s.name.clone()).collect();
        m["always_applied_skills"] = serde_json::json!(names);
    }
    Some(m)
}


