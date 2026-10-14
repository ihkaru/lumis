use std::sync::Arc;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use lumis_core::models::{EnterpriseRole, McpToolCallRequest, NewSession, User};
use lumis_core::traits::McpRegistry;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::mcp::{register_planner_tools, InMemoryMcpRegistry};
use lumis_server::routes::chat::executor_helpers::check_plan_convergence;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_dynamic_task_planner_mcp_and_convergence_lifecycle() {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Initialize SQLite in-memory driver");
    driver.run_migrations().await.expect("Run migrations");
    let repo = driver.repository();

    let guardrail = Arc::new(SimdGuardrailMatcher::new(vec![]).unwrap());
    let claim_translator = Arc::new(lumis_core::traits::DefaultClaimTranslator::new());
    let mcp_registry = Arc::new(InMemoryMcpRegistry::new());

    let state = AppState::with_options(
        driver,
        guardrail,
        claim_translator,
        Arc::new(lumis_server::finops::InMemoryQuotaEngine::new()),
        Arc::new(lumis_server::finops::PrometheusTelemetryCollector::new()),
        Arc::new(lumis_server::rag::HybridKnowledgeVault::new()),
        mcp_registry.clone(),
        Arc::new(lumis_server::audit::InMemoryAuditLogger::default()),
        "x-forwarded-user".to_string(),
    );

    // 1. Register planner tools
    register_planner_tools(mcp_registry.as_ref(), repo.clone(), state.stream_tx.clone())
        .await
        .unwrap();

    let tools = mcp_registry.list_tools(EnterpriseRole::Member).await;
    let tool_names: Vec<String> = tools.into_iter().map(|t| t.name).collect();
    assert!(tool_names.contains(&"plan_task_create".to_string()));
    assert!(tool_names.contains(&"plan_task_update".to_string()));
    assert!(tool_names.contains(&"plan_task_list".to_string()));
    assert!(tool_names.contains(&"plan_task_delete".to_string()));

    // 2. Setup user and session
    let user_id = Uuid::new_v4().to_string();
    let user = User {
        id: user_id.clone(),
        username: "dev_lead".to_string(),
        email: Some("lead@enterprise.com".to_string()),
        role: EnterpriseRole::Member,
        tenant_id: Some("dev-hq".to_string()),
        preferences: json!({}),
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
            title: "Build Microservice Task".to_string(),
            model_id: "deepseek-v3".to_string(),
            system_prompt: None,
        })
        .await
        .unwrap();

    // Subscribe to SSE stream to test live event emission
    let mut stream_rx = state.stream_tx.subscribe();

    // 3. Agent executes plan_task_create via MCP call
    let create_req = McpToolCallRequest {
        tool_name: "plan_task_create".to_string(),
        arguments: json!({
            "session_id": session.id,
            "title": "Setup Docker Compose configuration",
            "description": "Configure PostgreSQL and Valkey services",
            "order_index": 1
        }),
    };
    let res = mcp_registry.execute_tool(EnterpriseRole::Member, &create_req).await.unwrap();
    assert!(!res.is_error);
    let task1: serde_json::Value = serde_json::from_str(&res.content).unwrap();
    let task1_id = task1["id"].as_str().unwrap().to_string();
    assert_eq!(task1["status"], "pending");

    // Verify SSE event emitted
    let evt_msg = stream_rx.recv().await.unwrap();
    let evt_json: serde_json::Value = serde_json::from_slice(&evt_msg).unwrap();
    assert_eq!(evt_json["event"], "task_created");
    assert_eq!(evt_json["task"]["id"], task1_id);

    // Goal not converged yet
    assert!(!check_plan_convergence(&repo, &session.id).await);

    // 4. Agent creates second task
    let create_req2 = McpToolCallRequest {
        tool_name: "plan_task_create".to_string(),
        arguments: json!({
            "session_id": session.id,
            "title": "Run Integration Tests",
            "order_index": 2
        }),
    };
    let res2 = mcp_registry.execute_tool(EnterpriseRole::Member, &create_req2).await.unwrap();
    let task2: serde_json::Value = serde_json::from_str(&res2.content).unwrap();
    let task2_id = task2["id"].as_str().unwrap().to_string();

    // 5. Agent inspects plan via plan_task_list
    let list_req = McpToolCallRequest {
        tool_name: "plan_task_list".to_string(),
        arguments: json!({ "session_id": session.id }),
    };
    let list_res = mcp_registry.execute_tool(EnterpriseRole::Member, &list_req).await.unwrap();
    let listed_tasks: Vec<serde_json::Value> = serde_json::from_str(&list_res.content).unwrap();
    assert_eq!(listed_tasks.len(), 2);

    // 6. Agent completes task 1 and task 2
    let update_req1 = McpToolCallRequest {
        tool_name: "plan_task_update".to_string(),
        arguments: json!({
            "task_id": task1_id,
            "status": "completed",
            "result_summary": "Docker compose tested: exit code 0"
        }),
    };
    let up_res1 = mcp_registry.execute_tool(EnterpriseRole::Member, &update_req1).await.unwrap();
    assert!(!up_res1.is_error);

    // Still not converged because task 2 is pending
    assert!(!check_plan_convergence(&repo, &session.id).await);

    let update_req2 = McpToolCallRequest {
        tool_name: "plan_task_update".to_string(),
        arguments: json!({
            "task_id": task2_id,
            "status": "completed",
            "result_summary": "All 12 test assertions passed"
        }),
    };
    let _ = mcp_registry.execute_tool(EnterpriseRole::Member, &update_req2).await.unwrap();

    // Now all tasks are completed -> Goal Converged!
    assert!(check_plan_convergence(&repo, &session.id).await);

    // 7. Verify Axum REST API endpoints
    let app = create_router(state);

    // GET /api/sessions/{session_id}/tasks
    let req = Request::builder()
        .uri(format!("/api/sessions/{}/tasks", session.id))
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let api_tasks: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(api_tasks.len(), 2);
    assert_eq!(api_tasks[0]["status"], "completed");
    assert_eq!(api_tasks[1]["status"], "completed");
}
