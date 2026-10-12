use lumis_core::models::{CursorQuery, MessageRole, NewMessage, NewSession, User, EnterpriseRole};
use lumis_core::traits::ChatRepository;
use lumis_db::SqliteRepository;
use sqlx::SqlitePool;
use uuid::Uuid;

#[tokio::test]
async fn test_sqlite_repository_lifecycle() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let repo = SqliteRepository::new(pool).await.unwrap();

    // 1. Create User
    let user_id = Uuid::new_v4().to_string();
    let user = User {
        id: user_id.clone(),
        username: "test_ciso".to_string(),
        email: Some("ciso@enterprise.com".to_string()),
        role: EnterpriseRole::SuperAdmin,
        tenant_id: Some("hq-01".to_string()),
        preferences: serde_json::json!({ "theme": "dark" }),
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repo.upsert_user(&user).await.unwrap();

    let fetched_user = repo.get_user_by_id(&user_id).await.unwrap();
    assert!(fetched_user.is_some());
    assert_eq!(fetched_user.unwrap().role, EnterpriseRole::SuperAdmin);

    // 2. Create Session
    let session = repo
        .create_session(&NewSession {
            user_id: user_id.clone(),
            title: "Security Review".to_string(),
            model_id: "deepseek-r1".to_string(),
            system_prompt: Some("You are an enterprise auditor".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(session.title, "Security Review");

    // 3. Append Messages
    for i in 1..=5 {
        repo.append_message(&NewMessage {
            session_id: session.id.clone(),
            role: if i % 2 == 1 { MessageRole::User } else { MessageRole::Assistant },
            content: format!("Message content #{}", i),
            model_id: Some("deepseek-r1".to_string()),
            token_count: Some(15),
            metadata: None,
        })
        .await
        .unwrap();
    }

    // 4. Keyset (Cursor) Pagination with limit = 2
    let page1 = repo
        .list_messages(
            &session.id,
            &CursorQuery {
                cursor_time: None,
                cursor_id: None,
                limit: 2,
            },
        )
        .await
        .unwrap();

    assert_eq!(page1.items.len(), 2);
    assert!(page1.has_more);
    assert!(page1.next_cursor_time.is_some());
    assert!(page1.next_cursor_id.is_some());

    // Page 2 using cursor from Page 1
    let page2 = repo
        .list_messages(
            &session.id,
            &CursorQuery {
                cursor_time: page1.next_cursor_time,
                cursor_id: page1.next_cursor_id,
                limit: 2,
            },
        )
        .await
        .unwrap();

    assert_eq!(page2.items.len(), 2);
    assert!(page2.has_more);

    // Verify messages are different
    assert_ne!(page1.items[0].id, page2.items[0].id);
}
