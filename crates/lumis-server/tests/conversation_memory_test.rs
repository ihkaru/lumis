use std::sync::Arc;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use chrono::Utc;
use lumis_core::models::{
    EnterpriseRole, Message, MessageRole, Session, User, UserMemory,
};
use lumis_core::traits::DefaultClaimTranslator;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes::chat::assembly::{
    assemble_chat_context, extract_heuristic_title,
};
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use tower::ServiceExt;

async fn test_app() -> axum::Router {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Initialize SQLite in-memory driver");
    driver.run_migrations().await.expect("Run migrations");

    let guardrail = Arc::new(
        SimdGuardrailMatcher::new(vec!["sk-ant-".to_string(), "DROP TABLE".to_string()])
            .expect("Guardrail init"),
    );
    let claim_translator = Arc::new(DefaultClaimTranslator::new());
    let state = AppState::new(
        driver,
        guardrail,
        claim_translator,
        "x-forwarded-user".to_string(),
    );
    create_router(state)
}

#[tokio::test]
async fn test_multi_turn_context_assembly_order_and_system_prompt() {
    let user = User {
        id: "u_test_1".to_string(),
        username: "ahmad_bps".to_string(),
        email: Some("ahmad@bps.go.id".to_string()),
        role: EnterpriseRole::Member,
        tenant_id: Some("bps".to_string()),
        preferences: serde_json::json!({}),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let session = Session {
        id: "sess_1".to_string(),
        user_id: user.id.clone(),
        title: "Diskusi Data Sensus".to_string(),
        model_id: "my9model-free".to_string(),
        system_prompt: Some("Fokus pada data statistik wilayah Kalimantan Barat.".to_string()),
        is_pinned: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let history = vec![
        Message {
            id: "m_1".to_string(),
            session_id: session.id.clone(),
            role: MessageRole::User,
            content: "Berapa proyeksi penduduk Mempawah tahun 2026?".to_string(),
            model_id: Some("my9model-free".to_string()),
            token_count: Some(15),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        },
        Message {
            id: "m_2".to_string(),
            session_id: session.id.clone(),
            role: MessageRole::Assistant,
            content: "Proyeksi penduduk Mempawah tahun 2026 adalah sekitar 310.000 jiwa.".to_string(),
            model_id: Some("my9model-free".to_string()),
            token_count: Some(20),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        },
    ];

    let user_memories = vec![
        UserMemory {
            id: "mem_1".to_string(),
            user_id: user.id.clone(),
            category: "profile".to_string(),
            content: "Statistisi Ahli Muda di BPS Mempawah Satker 6104".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        UserMemory {
            id: "mem_2".to_string(),
            user_id: user.id.clone(),
            category: "preference".to_string(),
            content: "Gunakan bahasa Indonesia formal dan cantumkan tabel jika data berbentuk tabular.".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];

    let current_query = "Tampilkan rincian per kecamatan.";
    let assembled = assemble_chat_context(
        &session,
        &user,
        &history,
        current_query,
        &user_memories,
        None,
    );

    // Assert system prompt at index 0 contains base, custom session instructions, and user memories
    assert_eq!(assembled[0].role, "system");
    assert!(assembled[0].content.contains("You are Lumis"));
    assert!(assembled[0].content.contains("Fokus pada data statistik wilayah Kalimantan Barat."));
    assert!(assembled[0].content.contains("<user_memories>"));
    assert!(assembled[0].content.contains("Statistisi Ahli Muda di BPS Mempawah"));
    assert!(assembled[0].content.contains("Gunakan bahasa Indonesia formal"));

    // Assert history turns in chronological order
    assert_eq!(assembled[1].role, "user");
    assert_eq!(assembled[1].content, "Berapa proyeksi penduduk Mempawah tahun 2026?");

    assert_eq!(assembled[2].role, "assistant");
    assert_eq!(assembled[2].content, "Proyeksi penduduk Mempawah tahun 2026 adalah sekitar 310.000 jiwa.");

    // Assert tail dynamic context stamp
    let second_to_last = &assembled[assembled.len() - 2];
    assert_eq!(second_to_last.role, "system");
    assert!(second_to_last.content.contains("System Context: Current Time:"));
    assert!(second_to_last.content.contains("User: ahmad_bps"));

    // Assert final user prompt
    let last = assembled.last().unwrap();
    assert_eq!(last.role, "user");
    assert_eq!(last.content, current_query);
}

#[tokio::test]
async fn test_sliding_window_context_truncation_budget() {
    let user = User {
        id: "u_2".to_string(),
        username: "tester".to_string(),
        email: None,
        role: EnterpriseRole::Member,
        tenant_id: None,
        preferences: serde_json::json!({}),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let session = Session {
        id: "sess_2".to_string(),
        user_id: user.id.clone(),
        title: "Test Pruning".to_string(),
        model_id: "m".to_string(),
        system_prompt: None,
        is_pinned: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create 10 large messages
    let mut history = Vec::new();
    for i in 1..=10 {
        history.push(Message {
            id: format!("m_{}", i),
            session_id: session.id.clone(),
            role: if i % 2 == 1 { MessageRole::User } else { MessageRole::Assistant },
            content: format!("Pesan nomor {} dengan teks yang panjang: {}", i, "x".repeat(300)),
            model_id: None,
            token_count: Some(80),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        });
    }

    // Set a very tight budget (e.g. 350 tokens)
    let assembled = assemble_chat_context(
        &session,
        &user,
        &history,
        "Pertanyaan terakhir",
        &[],
        Some(350),
    );

    // Oldest messages should be pruned
    assert!(assembled.iter().any(|m| m.content.contains("pruned to fit context window")));
    // Most recent history message (m_10) should be preserved
    assert!(assembled.iter().any(|m| m.content.contains("Pesan nomor 10")));
    // Oldest message (m_1) should be pruned
    assert!(!assembled.iter().any(|m| m.content.contains("Pesan nomor 1 dengan teks")));
}

#[tokio::test]
async fn test_heuristic_title_extraction() {
    let title1 = extract_heuristic_title("Bantu buatkan analisis perbandingan inflasi antar provinsi di Indonesia");
    assert_eq!(title1, "Bantu buatkan analisis perbandingan");

    let title2 = extract_heuristic_title("```python\nprint('hello')\n```");
    assert_eq!(title2, "python print( hello )");

    let title3 = extract_heuristic_title("   ");
    assert_eq!(title3, "Percakapan Baru");
}

#[tokio::test]
async fn test_user_memory_crud_api_and_chat_integration() {
    let app = test_app().await;

    // 1. Initially empty
    let req = Request::builder()
        .uri("/api/user/memories")
        .method("GET")
        .header("x-forwarded-user", "budi_bps")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["memories"].as_array().unwrap().len(), 0);

    // 2. Create memory
    let req = Request::builder()
        .uri("/api/user/memories")
        .method("POST")
        .header("x-forwarded-user", "budi_bps")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&serde_json::json!({
            "category": "role",
            "content": "Ketua Tim Nerwilis BPS Mempawah"
        })).unwrap()))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let created_mem: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let mem_id = created_mem["id"].as_str().unwrap();

    // 3. List again - contains created memory
    let req = Request::builder()
        .uri("/api/user/memories")
        .method("GET")
        .header("x-forwarded-user", "budi_bps")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["memories"].as_array().unwrap().len(), 1);
    assert_eq!(json["memories"][0]["content"], "Ketua Tim Nerwilis BPS Mempawah");

    // 4. Delete memory
    let req = Request::builder()
        .uri(format!("/api/user/memories/{}", mem_id))
        .method("DELETE")
        .header("x-forwarded-user", "budi_bps")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 5. Verify deleted
    let req = Request::builder()
        .uri("/api/user/memories")
        .method("GET")
        .header("x-forwarded-user", "budi_bps")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["memories"].as_array().unwrap().len(), 0);
}
