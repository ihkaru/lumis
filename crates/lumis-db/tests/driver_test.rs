use chrono::Utc;
use lumis_core::models::{
    CursorQuery, EnterpriseRole, MessageRole, NewMessage, NewSession, User,
};
use lumis_core::traits::SqlDialect;
use lumis_db::DatabaseFactory;
use uuid::Uuid;

#[tokio::test]
async fn test_relational_driver_sqlite_in_memory() {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Failed to initialize SQLite driver");

    // 1. Dialect Verification
    assert_eq!(driver.dialect(), SqlDialect::Sqlite);
    assert_eq!(driver.dialect().as_str(), "sqlite");
    assert_eq!(driver.dialect().placeholder(1), "?");

    // 2. Ping Verification
    driver.ping().await.expect("Driver ping must succeed");

    // 3. Health Verification
    let health = driver.health().await.expect("Health check must succeed");
    assert_eq!(health.dialect, SqlDialect::Sqlite);
    assert!(health.is_alive);
    assert!(health.active_connections >= 1);

    // 4. Schema Migration Idempotency
    driver
        .run_migrations()
        .await
        .expect("Schema migrations must be idempotent");

    // 5. Repository Verification through Driver Abstraction (DIP)
    let repo = driver.repository();
    let now = Utc::now();
    let test_user = User {
        id: Uuid::new_v4().to_string(),
        username: "driver_test_user".to_string(),
        email: Some("driver@example.com".to_string()),
        role: EnterpriseRole::PowerUser,
        tenant_id: Some("tenant-1".to_string()),
        preferences: serde_json::json!({"theme": "dark"}),
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    let created_user = repo.upsert_user(&test_user).await.expect("Upsert user");
    assert_eq!(created_user.username, "driver_test_user");
    assert_eq!(created_user.role, EnterpriseRole::PowerUser);

    let session = repo
        .create_session(&NewSession {
            user_id: created_user.id.clone(),
            title: "Driver Test Session".to_string(),
            model_id: "deepseek-r1".to_string(),
            system_prompt: None,
        })
        .await
        .expect("Create session");

    // Keyset pagination verification
    for i in 1..=5 {
        repo.append_message(&NewMessage {
            session_id: session.id.clone(),
            role: MessageRole::User,
            content: format!("Message index {}", i),
            model_id: Some("deepseek-r1".to_string()),
            token_count: Some(10),
            metadata: None,
        })
        .await
        .expect("Append message");
    }

    let page = repo
        .list_messages(&session.id, &CursorQuery { cursor_time: None, cursor_id: None, limit: 3 })
        .await
        .expect("List messages page");

    assert_eq!(page.items.len(), 3);
    assert!(page.has_more);
    assert!(page.next_cursor_time.is_some());
    assert!(page.next_cursor_id.is_some());
}
