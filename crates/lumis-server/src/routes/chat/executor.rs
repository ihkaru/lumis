use std::collections::HashMap;
use lumis_core::models::{
    ChatCompletionRequest, ChatMessageFunctionCall, ChatMessagePayload, ChatMessageToolCall,
    CursorQuery, McpToolCallRequest, MessageRole, NewMessage, User,
};
use serde_json::json;
use crate::state::AppState;
use super::assembly::{assemble_chat_context, extract_heuristic_title, format_mcp_tools_for_openai};

const MAX_AGENTIC_TURNS: usize = 5;

fn parse_text_tool_calls(text: &str) -> Vec<(String, String, serde_json::Value)> {
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

pub fn spawn_chat_completion(
    state: AppState,
    session_id: String,
    user: User,
    model: String,
    prompt: String,
) {
    let router = match state.llm_router.clone() {
        Some(r) => r,
        None => return,
    };

    tokio::spawn(async move {
        let stream_tx = state.stream_tx.clone();
        let repo = state.repo.clone();
        let sess_id = session_id.clone();

        let session = match repo.get_session(&sess_id).await {
            Ok(Some(s)) => s,
            _ => return,
        };

        let query = CursorQuery { cursor_time: None, cursor_id: None, limit: 40 };
        let mut past_messages = repo.list_messages(&sess_id, &query).await.map(|p| p.items).unwrap_or_default();
        past_messages.reverse();

        let user_memories = repo.list_user_memories(&user.id).await.unwrap_or_default();
        let active_tools = state.mcp_registry.list_tools(user.role).await;
        let openai_tools = if !active_tools.is_empty() {
            Some(format_mcp_tools_for_openai(&active_tools))
        } else {
            None
        };

        let mut messages = assemble_chat_context(
            &session,
            &user,
            &past_messages,
            &prompt,
            &model,
            &user_memories,
            Some(&active_tools),
            None,
        );

        let mut executed_tools = Vec::new();
        let mut final_text = String::new();

        for _turn in 0..MAX_AGENTIC_TURNS {
            let req = ChatCompletionRequest {
                model: model.clone(),
                messages: messages.clone(),
                temperature: Some(0.7),
                max_tokens: None,
                stream: true,
                tools: openai_tools.clone(),
                tool_choice: None,
            };

            let mut rx = match router.complete_chat(&req).await {
                Ok(receiver) => receiver,
                Err(e) => {
                    tracing::error!("LLM completion turn failed: {}", e);
                    let err_event = json!({ "event": "error", "session_id": sess_id, "error": e.to_string() });
                    let _ = stream_tx.send(bytes::Bytes::from(err_event.to_string()));
                    return;
                }
            };

            let mut turn_text = String::new();
            let mut native_tcs: HashMap<usize, (Option<String>, Option<String>, String)> = HashMap::new();

            while let Some(chunk_res) = rx.recv().await {
                match chunk_res {
                    Ok(chunk) => {
                        if let Some(tcs) = chunk.tool_calls {
                            for tc in tcs {
                                let entry = native_tcs.entry(tc.index).or_insert((None, None, String::new()));
                                if tc.id.is_some() { entry.0 = tc.id; }
                                if tc.name.is_some() { entry.1 = tc.name; }
                                if let Some(arg_delta) = tc.arguments { entry.2.push_str(&arg_delta); }
                            }
                        }
                        if !chunk.content.is_empty() {
                            turn_text.push_str(&chunk.content);
                            if !turn_text.contains("<tool_call") && native_tcs.is_empty() {
                                let chunk_event = json!({
                                    "event": "chunk",
                                    "session_id": sess_id,
                                    "delta": chunk.content,
                                    "model_id": model,
                                    "is_done": false,
                                });
                                let _ = stream_tx.send(bytes::Bytes::from(chunk_event.to_string()));
                            }
                        }
                        if chunk.is_done { break; }
                    }
                    Err(e) => {
                        tracing::error!("Error receiving chunk: {}", e);
                        break;
                    }
                }
            }

            let mut detected_calls = Vec::new();
            if !native_tcs.is_empty() {
                let mut sorted_indices: Vec<usize> = native_tcs.keys().cloned().collect();
                sorted_indices.sort();
                for idx in sorted_indices {
                    if let Some((id_opt, name_opt, arg_str)) = native_tcs.get(&idx) {
                        if let Some(name) = name_opt {
                            let cid = id_opt.clone().unwrap_or_else(|| format!("call_{}", uuid::Uuid::new_v4().simple()));
                            let args_val = serde_json::from_str(arg_str).unwrap_or(json!({}));
                            detected_calls.push((cid, name.clone(), args_val));
                        }
                    }
                }
            } else if turn_text.contains("<tool_call>") {
                detected_calls = parse_text_tool_calls(&turn_text);
            }

            if detected_calls.is_empty() {
                final_text = turn_text;
                let done_event = json!({
                    "event": "chunk",
                    "session_id": sess_id,
                    "delta": "",
                    "model_id": model,
                    "is_done": true,
                });
                let _ = stream_tx.send(bytes::Bytes::from(done_event.to_string()));
                break;
            }

            for (cid, tool_name, args) in detected_calls {
                let start_event = json!({
                    "event": "tool_call_start",
                    "session_id": sess_id,
                    "call_id": cid,
                    "tool": tool_name,
                    "args": args,
                });
                let _ = stream_tx.send(bytes::Bytes::from(start_event.to_string()));

                let scrubbed_args = state.hook_pipeline.execute_before(&user, &tool_name, &args).await.unwrap_or_else(|_| args.clone());
                let tool_req = McpToolCallRequest { tool_name: tool_name.clone(), arguments: scrubbed_args };
                let exec_result = state.mcp_registry.execute_tool(user.role, &tool_req).await;

                let (output_str, is_err) = match exec_result {
                    Ok(res) => {
                        let audited = state.hook_pipeline.execute_after(&user, &tool_name, &serde_json::Value::String(res.content.clone())).await
                            .ok().and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or(res.content);
                        (audited, res.is_error)
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        state.hook_pipeline.execute_on_error(&user, &tool_name, &err_msg).await;
                        (format!("Error: {}", err_msg), true)
                    }
                };

                let res_event = json!({
                    "event": "tool_call_result",
                    "session_id": sess_id,
                    "call_id": cid,
                    "tool": tool_name,
                    "result": output_str,
                    "is_error": is_err,
                });
                let _ = stream_tx.send(bytes::Bytes::from(res_event.to_string()));

                executed_tools.push(json!({
                    "tool": tool_name,
                    "args": args,
                    "result": output_str,
                    "error": if is_err { Some(&output_str) } else { None },
                }));

                messages.push(ChatMessagePayload {
                    role: "assistant".to_string(),
                    content: turn_text.clone(),
                    tool_calls: Some(vec![ChatMessageToolCall {
                        id: cid.clone(),
                        r#type: "function".to_string(),
                        function: ChatMessageFunctionCall {
                            name: tool_name.clone(),
                            arguments: args.to_string(),
                        },
                    }]),
                    tool_call_id: None,
                });

                messages.push(ChatMessagePayload {
                    role: "tool".to_string(),
                    content: output_str.clone(),
                    tool_calls: None,
                    tool_call_id: Some(cid),
                });

                messages.push(ChatMessagePayload {
                    role: "user".to_string(),
                    content: format!("[System Tool Output for '{}']: {}\nPlease answer the user's initial question using this data.", tool_name, output_str),
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
        }

        if !final_text.is_empty() {
            let metadata = if !executed_tools.is_empty() {
                Some(json!({ "tool_calls": executed_tools }))
            } else {
                None
            };
            let assistant_msg = NewMessage {
                session_id: sess_id.clone(),
                role: MessageRole::Assistant,
                content: final_text,
                model_id: Some(model),
                token_count: None,
                metadata,
            };
            let _ = repo.append_message(&assistant_msg).await;

            if session.title == "Percakapan Baru" || session.title.trim().is_empty() {
                let new_title = extract_heuristic_title(&prompt);
                if let Ok(true) = repo.update_session_title(&sess_id, &new_title).await {
                    let title_event = json!({
                        "event": "title",
                        "session_id": sess_id,
                        "title": new_title,
                    });
                    let _ = stream_tx.send(bytes::Bytes::from(title_event.to_string()));
                }
            }
        }
    });
}
