use lumis_core::models::{CursorQuery, MessageRole, NewMessage, NewSession, User, EnterpriseRole};
use lumis_db::DatabaseFactory;
use uuid::Uuid;

#[tokio::test]
async fn test_sqlite_repository_lifecycle() {
    let repo = DatabaseFactory::create_repository("sqlite::memory:")
        .await
        .expect("Failed to initialize SQLite repository");

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

#[tokio::test]
async fn test_sqlite_session_task_state_machine() {
    let repo = DatabaseFactory::create_repository("sqlite::memory:")
        .await
        .expect("Failed to initialize SQLite repository");

    let user_id = Uuid::new_v4().to_string();
    let user = User {
        id: user_id.clone(),
        username: "developer_agent".to_string(),
        email: Some("agent@enterprise.com".to_string()),
        role: EnterpriseRole::Member,
        tenant_id: Some("dev-cluster".to_string()),
        preferences: serde_json::json!({}),
        is_active: true,
        password_hash: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repo.upsert_user(&user).await.unwrap();

    let session = repo
        .create_session(&NewSession {
            id: None,
            user_id: user_id.clone(),
            title: "Autonomous Coding Plan".to_string(),
            model_id: "deepseek-coder".to_string(),
            system_prompt: None,
        })
        .await
        .unwrap();

    // 1. Create baseline plan tasks
    let t1 = repo
        .create_session_task(&lumis_core::models::CreateTaskInput {
            id: Some("task-explore".to_string()),
            session_id: session.id.clone(),
            parent_task_id: None,
            title: "Inspect codebase architecture".to_string(),
            description: Some("Run code-graph and check module dependencies".to_string()),
            status: lumis_core::models::TaskStatus::Pending,
            order_index: Some(1),
        })
        .await
        .unwrap();
    assert_eq!(t1.id, "task-explore");
    assert_eq!(t1.status, lumis_core::models::TaskStatus::Pending);

    let _t2 = repo
        .create_session_task(&lumis_core::models::CreateTaskInput {
            id: Some("task-implement".to_string()),
            session_id: session.id.clone(),
            parent_task_id: None,
            title: "Implement core algorithm".to_string(),
            description: None,
            status: lumis_core::models::TaskStatus::Pending,
            order_index: Some(2),
        })
        .await
        .unwrap();

    // 2. Add dynamic subtask discovered during execution
    let t1_sub = repo
        .create_session_task(&lumis_core::models::CreateTaskInput {
            id: Some("task-explore-sub1".to_string()),
            session_id: session.id.clone(),
            parent_task_id: Some(t1.id.clone()),
            title: "Parse AST tree nodes".to_string(),
            description: Some("Subtask for AST traversal".to_string()),
            status: lumis_core::models::TaskStatus::InProgress,
            order_index: Some(3),
        })
        .await
        .unwrap();
    assert_eq!(t1_sub.parent_task_id.as_deref(), Some("task-explore"));

    // 3. List tasks and verify ordering
    let tasks = repo.list_session_tasks(&session.id).await.unwrap();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].id, "task-explore");
    assert_eq!(tasks[1].id, "task-implement");
    assert_eq!(tasks[2].id, "task-explore-sub1");

    // 4. Update task status with empirical verification evidence
    let updated = repo
        .update_session_task(
            "task-explore-sub1",
            &lumis_core::models::UpdateTaskInput {
                title: None,
                description: None,
                status: Some(lumis_core::models::TaskStatus::Completed),
                order_index: None,
                result_summary: Some("AST nodes verified: 42 symbols indexed".to_string()),
            },
        )
        .await
        .unwrap();
    assert!(updated.is_some());
    let u = updated.unwrap();
    assert_eq!(u.status, lumis_core::models::TaskStatus::Completed);
    assert_eq!(u.result_summary.as_deref(), Some("AST nodes verified: 42 symbols indexed"));

    // 5. Delete single task
    let deleted = repo.delete_session_task("task-explore-sub1").await.unwrap();
    assert!(deleted);
    let tasks_after_del = repo.list_session_tasks(&session.id).await.unwrap();
    assert_eq!(tasks_after_del.len(), 2);

    // 6. Clear all tasks
    let cleared = repo.clear_session_tasks(&session.id).await.unwrap();
    assert_eq!(cleared, 2);
    let tasks_empty = repo.list_session_tasks(&session.id).await.unwrap();
    assert_eq!(tasks_empty.len(), 0);
}
