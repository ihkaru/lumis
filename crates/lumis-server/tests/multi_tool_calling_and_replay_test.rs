use chrono::Utc;
use lumis_core::models::{
    ChatMessagePayload, EnterpriseRole, Message, MessageRole, Session, User,
};
use lumis_server::routes::chat::assembly::assemble_chat_context;

#[tokio::test]
async fn test_multi_tool_history_replay_matches_openai_spec() {
    let user = User {
        id: "u_test_agent".to_string(),
        username: "analyst_bps".to_string(),
        email: Some("analyst@bps.go.id".to_string()),
        role: EnterpriseRole::Member,
        tenant_id: Some("bps".to_string()),
        preferences: serde_json::json!({}),
        is_active: true,
        password_hash: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let session = Session {
        id: "sess_agent_1".to_string(),
        user_id: user.id.clone(),
        title: "Pencarian Data BPS".to_string(),
        model_id: "cbai/minimax-m3".to_string(),
        system_prompt: None,
        is_pinned: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Simulate turn 1: User prompt
    // Turn 2: Assistant executes 2 web searches and provides final answer
    let history = vec![
        Message {
            id: "m_user_1".to_string(),
            session_id: session.id.clone(),
            role: MessageRole::User,
            content: "Cari website BPS Mempawah dan batas wilayahnya.".to_string(),
            model_id: Some("cbai/minimax-m3".to_string()),
            token_count: Some(15),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        },
        Message {
            id: "m_asst_1".to_string(),
            session_id: session.id.clone(),
            role: MessageRole::Assistant,
            content: "Berdasarkan hasil pencarian, website resmi adalah mempawahkab.bps.go.id.".to_string(),
            model_id: Some("cbai/minimax-m3".to_string()),
            token_count: Some(50),
            metadata: serde_json::json!({
                "pre_content": "Saya akan mencari informasi tersebut di internet.",
                "tool_calls": [
                    {
                        "call_id": "call_search_1",
                        "tool": "gateway_web_search",
                        "args": { "query": "website bps mempawah" },
                        "result": "Domain resmi: mempawahkab.bps.go.id",
                        "error": null
                    },
                    {
                        "call_id": "call_search_2",
                        "tool": "gateway_web_search",
                        "args": { "query": "batas wilayah mempawah" },
                        "result": "Batas: Utara Kab Bengkayang, Selatan Kab Kubu Raya",
                        "error": null
                    }
                ],
                "parts": [
                    { "type": "text", "text": "Saya akan mencari informasi tersebut di internet." },
                    { "type": "tool_call", "call_id": "call_search_1", "tool": "gateway_web_search", "args": { "query": "website bps mempawah" }, "result": "Domain resmi: mempawahkab.bps.go.id" },
                    { "type": "tool_call", "call_id": "call_search_2", "tool": "gateway_web_search", "args": { "query": "batas wilayah mempawah" }, "result": "Batas: Utara Kab Bengkayang, Selatan Kab Kubu Raya" },
                    { "type": "text", "text": "Berdasarkan hasil pencarian, website resmi adalah mempawahkab.bps.go.id." }
                ]
            }),
            created_at: Utc::now(),
        },
    ];

    // Reconstruct context for turn 3
    let next_prompt = "Bagaimana dengan jumlah penduduknya?";
    let messages = assemble_chat_context(
        &session,
        &user,
        &history,
        next_prompt,
        "cbai/minimax-m3",
        &[],
        None,
        None,
        None,
        None,
    );

    // Verify System Prompt and Dynamic context stamp
    assert_eq!(messages[0].role, "system");

    // History verification
    // Must find:
    // 1. user message: "Cari website BPS Mempawah..."
    let user_msg = messages.iter().find(|m| m.role == "user" && m.content.contains("Cari website BPS")).expect("User message present");
    assert_eq!(user_msg.content, "Cari website BPS Mempawah dan batas wilayahnya.");

    // 2. assistant message with tool_calls: [call_search_1, call_search_2]
    let tool_call_asst = messages.iter().find(|m| m.role == "assistant" && m.tool_calls.is_some()).expect("Assistant tool call turn present");
    let tcs = tool_call_asst.tool_calls.as_ref().unwrap();
    assert_eq!(tcs.len(), 2, "Both tool calls must be present in single assistant message");
    assert_eq!(tcs[0].id, "call_search_1");
    assert_eq!(tcs[0].function.name, "gateway_web_search");
    assert_eq!(tcs[1].id, "call_search_2");
    assert_eq!(tcs[1].function.name, "gateway_web_search");

    // 3. Corresponding tool messages
    let tool_msgs: Vec<&ChatMessagePayload> = messages.iter().filter(|m| m.role == "tool").collect();
    assert_eq!(tool_msgs.len(), 2, "Must contain exactly 2 tool result messages");
    assert_eq!(tool_msgs[0].tool_call_id.as_deref(), Some("call_search_1"));
    assert!(tool_msgs[0].content.contains("mempawahkab.bps.go.id"));
    assert_eq!(tool_msgs[1].tool_call_id.as_deref(), Some("call_search_2"));
    assert!(tool_msgs[1].content.contains("Bengkayang"));

    // 4. Final assistant answer text
    let final_asst = messages.iter().find(|m| m.role == "assistant" && m.tool_calls.is_none() && m.content.contains("Berdasarkan hasil pencarian")).expect("Final assistant answer present");
    assert!(final_asst.content.contains("mempawahkab.bps.go.id"));

    // 5. Zero interleaved fake user messages
    let fake_user_msgs: Vec<&ChatMessagePayload> = messages.iter().filter(|m| m.role == "user" && m.content.contains("[System Tool Output")).collect();
    assert_eq!(fake_user_msgs.len(), 0, "No fake user messages allowed in OpenAI tool calling format");

    // 6. Current user prompt at tail
    let tail_user = messages.last().expect("Tail user prompt");
    assert_eq!(tail_user.role, "user");
    assert_eq!(tail_user.content, next_prompt);
}

#[tokio::test]
async fn test_arbitrary_n_tools_calling_and_replay() {
    let user = User {
        id: "u_batch_tools".to_string(),
        username: "batch_tester".to_string(),
        email: None,
        role: EnterpriseRole::Member,
        tenant_id: Some("default".to_string()),
        preferences: serde_json::json!({}),
        is_active: true,
        password_hash: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let session = Session {
        id: "sess_batch_n".to_string(),
        user_id: user.id.clone(),
        title: "Test Arbitrary N Tools".to_string(),
        model_id: "cbai/minimax-m3".to_string(),
        system_prompt: None,
        is_pinned: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Construct an assistant turn with N = 8 distinct tool calls
    let n = 8;
    let mut tool_calls = Vec::new();
    let mut parts = Vec::new();
    parts.push(serde_json::json!({ "type": "text", "text": "Memulai pencarian 8 topik data paralel..." }));

    for i in 1..=n {
        let call_id = format!("call_batch_{}", i);
        let tool_name = format!("tool_service_{}", i);
        let query = format!("Kategori data ke-{}", i);
        let result = format!("Hasil data metrik ke-{}", i);

        tool_calls.push(serde_json::json!({
            "call_id": call_id.clone(),
            "tool": tool_name.clone(),
            "args": { "query": query },
            "result": result.clone(),
            "error": null
        }));

        parts.push(serde_json::json!({
            "type": "tool_call",
            "call_id": call_id,
            "tool": tool_name,
            "args": { "query": query },
            "result": result
        }));
    }

    parts.push(serde_json::json!({ "type": "text", "text": "Semua 8 data berhasil dihimpun." }));

    let history = vec![
        Message {
            id: "m_user_batch".to_string(),
            session_id: session.id.clone(),
            role: MessageRole::User,
            content: "Ambil 8 data statistik secara paralel.".to_string(),
            model_id: Some("cbai/minimax-m3".to_string()),
            token_count: Some(20),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        },
        Message {
            id: "m_asst_batch".to_string(),
            session_id: session.id.clone(),
            role: MessageRole::Assistant,
            content: "Semua 8 data berhasil dihimpun.".to_string(),
            model_id: Some("cbai/minimax-m3".to_string()),
            token_count: Some(200),
            metadata: serde_json::json!({
                "pre_content": "Memulai pencarian 8 topik data paralel...",
                "tool_calls": tool_calls,
                "parts": parts
            }),
            created_at: Utc::now(),
        },
    ];

    let messages = assemble_chat_context(
        &session,
        &user,
        &history,
        "Analisis tren dari data tersebut",
        "cbai/minimax-m3",
        &[],
        None,
        None,
        None,
        None,
    );

    // 1. Single assistant message containing ALL 8 tool_calls
    let asst_tool = messages.iter().find(|m| m.role == "assistant" && m.tool_calls.is_some()).expect("Batch tool assistant message present");
    let tcs = asst_tool.tool_calls.as_ref().unwrap();
    assert_eq!(tcs.len(), n, "Must contain all N tool calls in the same assistant turn");
    for i in 1..=n {
        assert_eq!(tcs[i - 1].id, format!("call_batch_{}", i));
        assert_eq!(tcs[i - 1].function.name, format!("tool_service_{}", i));
    }

    // 2. Exactly N corresponding tool result messages in order
    let tool_msgs: Vec<&ChatMessagePayload> = messages.iter().filter(|m| m.role == "tool").collect();
    assert_eq!(tool_msgs.len(), n, "Must have exactly N tool result messages");
    for i in 1..=n {
        assert_eq!(tool_msgs[i - 1].tool_call_id.as_deref(), Some(format!("call_batch_{}", i).as_str()));
        assert!(tool_msgs[i - 1].content.contains(&format!("Hasil data metrik ke-{}", i)));
    }

    // 3. Final assistant synthesis text
    let final_asst = messages.iter().find(|m| m.role == "assistant" && m.tool_calls.is_none() && m.content == "Semua 8 data berhasil dihimpun.").expect("Final assistant message present");
    assert_eq!(final_asst.content, "Semua 8 data berhasil dihimpun.");
}

