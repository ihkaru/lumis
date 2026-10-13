use lumis_core::models::{
    ChatCompletionRequest, CursorQuery, MessageRole, NewMessage, User,
};

use crate::state::AppState;
use super::assembly::{assemble_chat_context, extract_heuristic_title};

/// Spawns background streaming chat completion with full multi-turn context,
/// sliding window budgeting, persistent user memories, and auto-title generation.
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

        // 1. Fetch Session metadata
        let session = match repo.get_session(&sess_id).await {
            Ok(Some(s)) => s,
            _ => return,
        };

        // 2. Fetch past conversation turns (chronological order)
        let query = CursorQuery {
            cursor_time: None,
            cursor_id: None,
            limit: 40,
        };
        let mut past_messages = match repo.list_messages(&sess_id, &query).await {
            Ok(page) => page.items,
            Err(_) => Vec::new(),
        };
        // list_messages returns DESC order, reverse to ASC
        past_messages.reverse();

        // 3. Fetch persistent user memories
        let user_memories = repo.list_user_memories(&user.id).await.unwrap_or_default();

        // 4. Assemble multi-turn context (with budget pruning & tail temporal stamp)
        let messages = assemble_chat_context(
            &session,
            &user,
            &past_messages,
            &prompt,
            &model,
            &user_memories,
            None,
        );

        let req = ChatCompletionRequest {
            model: model.clone(),
            messages,
            temperature: Some(0.7),
            max_tokens: None,
            stream: true,
        };

        match router.complete_chat(&req).await {
            Ok(mut rx) => {
                let mut accumulated = String::new();
                while let Some(chunk_res) = rx.recv().await {
                    match chunk_res {
                        Ok(chunk) => {
                            accumulated.push_str(&chunk.content);
                            let chunk_event = serde_json::json!({
                                "event": "chunk",
                                "session_id": sess_id,
                                "delta": chunk.content,
                                "model_id": model,
                                "is_done": chunk.is_done,
                            });
                            let _ = stream_tx.send(bytes::Bytes::from(chunk_event.to_string()));
                            if chunk.is_done {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error receiving chunk from LLM router: {}", e);
                            break;
                        }
                    }
                }

                if !accumulated.is_empty() {
                    let assistant_msg = NewMessage {
                        session_id: sess_id.clone(),
                        role: MessageRole::Assistant,
                        content: accumulated,
                        model_id: Some(model),
                        token_count: None,
                        metadata: None,
                    };
                    let _ = repo.append_message(&assistant_msg).await;

                    // Auto Title Generation on First Turn (Llumen parity)
                    if session.title == "Percakapan Baru" || session.title.trim().is_empty() {
                        let new_title = extract_heuristic_title(&prompt);
                        if let Ok(true) = repo.update_session_title(&sess_id, &new_title).await {
                            let title_event = serde_json::json!({
                                "event": "title",
                                "session_id": sess_id,
                                "title": new_title,
                            });
                            let _ = stream_tx.send(bytes::Bytes::from(title_event.to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("LLM completion failed: {}", e);
                let err_event = serde_json::json!({
                    "event": "error",
                    "session_id": sess_id,
                    "error": e.to_string(),
                });
                let _ = stream_tx.send(bytes::Bytes::from(err_event.to_string()));
            }
        }
    });
}
