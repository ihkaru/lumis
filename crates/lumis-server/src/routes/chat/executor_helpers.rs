#![deny(unsafe_code)]

use std::sync::Arc;
use lumis_core::traits::ChatRepository;
use serde_json::json;

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
    if text.len() <= max_len {
        text.to_string()
    } else {
        let half = max_len / 2;
        format!(
            "{}... [truncated {} characters] ...{}",
            &text[..half],
            text.len() - max_len,
            &text[text.len() - half..]
        )
    }
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
                if let Some(name) = val["name"].as_str() {
                    let args = val.get("arguments").cloned().unwrap_or(json!({}));
                    let call_id = format!("call_{}", uuid::Uuid::new_v4().simple());
                    results.push((call_id, name.to_string(), args));
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
