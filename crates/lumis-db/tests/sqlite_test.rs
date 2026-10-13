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
        password_hash: None,
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
            id: None,
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

    // 5. Test Federated Identity Operations (Phase 2 Multi-IdP)
    let identity = lumis_core::models::UserIdentity {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.clone(),
        provider: "saml".to_string(),
        provider_subject_id: "199001012015011001".to_string(), // NIP BPS
        credential_data: Some(serde_json::json!({ "idp": "rIDM_BPS", "email": "ciso@enterprise.com" })),
        last_login_at: Some(chrono::Utc::now()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repo.link_identity(&identity).await.unwrap();

    let fetched_idty = repo.find_identity("saml", "199001012015011001").await.unwrap();
    assert!(fetched_idty.is_some());
    let idty = fetched_idty.unwrap();
    assert_eq!(idty.user_id, user_id);
    assert_eq!(idty.provider, "saml");

    let user_idties = repo.list_identities_by_user(&user_id).await.unwrap();
    assert_eq!(user_idties.len(), 1);

    let unlinked = repo.unlink_identity(&user_id, "saml").await.unwrap();
    assert!(unlinked);
    let after_unlink = repo.find_identity("saml", "199001012015011001").await.unwrap();
    assert!(after_unlink.is_none());
}
