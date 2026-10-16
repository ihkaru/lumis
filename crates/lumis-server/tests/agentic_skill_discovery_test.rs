use std::sync::Arc;
use lumis_core::models::{CreateSkillInput, EnterpriseRole, McpToolCallRequest};
use lumis_core::traits::McpRegistry;
use lumis_db::DatabaseFactory;
use lumis_server::mcp::{register_skill_tools, InMemoryMcpRegistry};
use lumis_server::routes::chat::executor_helpers::heal_or_infer_tool_name;
use serde_json::json;

#[tokio::test]
async fn test_agentic_skill_discovery_tools_lifecycle() {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Initialize SQLite in-memory driver");
    driver.run_migrations().await.expect("Run migrations");
    let repo = driver.repository();

    let mcp_registry = Arc::new(InMemoryMcpRegistry::new());

    // 1. Register skill discovery tools
    register_skill_tools(mcp_registry.as_ref(), repo.clone())
        .await
        .expect("Register skill tools");

    let tools = mcp_registry.list_tools(EnterpriseRole::Member).await;
    assert!(tools.iter().any(|t| t.name == "search_skills"), "search_skills must be registered");
    assert!(tools.iter().any(|t| t.name == "get_skill_directive"), "get_skill_directive must be registered");

    // 2. Seed test admin user and skills in database
    let admin_user = lumis_core::models::User {
        id: "admin".to_string(),
        username: "admin".to_string(),
        email: Some("admin@lumis.local".to_string()),
        role: EnterpriseRole::TenantAdmin,
        tenant_id: Some("default".to_string()),
        preferences: json!({}),
        is_active: true,
        password_hash: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repo.upsert_user(&admin_user).await.expect("Upsert admin user");

    let skill1 = CreateSkillInput {
        name: "bps-stat-methodology".to_string(),
        display_title: Some("Metodologi Statistik BPS".to_string()),
        description: "Standar pengolahan data statistik dan sampling error BPS.".to_string(),
        body: "# Panduan Metodologi Statistik\n1. Pastikan sampling frame valid.\n2. Hitung Margin of Error.".to_string(),
        frontmatter: None,
        category: Some("statistics".to_string()),
        always_apply: Some(true),
        user_invocable: Some(true),
        disable_model_invocation: Some(false),
        allowed_tools: None,
    };
    repo.create_skill("default", "admin", &skill1).await.expect("Create skill 1");

    let skill2 = CreateSkillInput {
        name: "spip-compliance".to_string(),
        display_title: Some("Kepatuhan SPIP BPS".to_string()),
        description: "Pedoman kepatuhan SPIP dan manajemen risiko tata kelola.".to_string(),
        body: "# Pedoman SPIP\nAudit kontrol internal sebelum diseminasi data.".to_string(),
        frontmatter: None,
        category: Some("compliance".to_string()),
        always_apply: Some(false),
        user_invocable: Some(true),
        disable_model_invocation: Some(false),
        allowed_tools: None,
    };
    repo.create_skill("default", "admin", &skill2).await.expect("Create skill 2");

    // 3. Test search_skills tool
    let search_req = McpToolCallRequest {
        tool_name: "search_skills".to_string(),
        arguments: json!({ "query": "statistik", "limit": 5 }),
    };
    let search_res = mcp_registry.execute_tool(EnterpriseRole::Member, &search_req).await.expect("Execute search_skills");
    assert!(!search_res.is_error);
    let search_json: serde_json::Value = serde_json::from_str(&search_res.content).expect("Valid JSON");
    assert_eq!(search_json["count"], 1);
    assert_eq!(search_json["skills"][0]["name"], "bps-stat-methodology");

    // 4. Test get_skill_directive tool with exact name
    let get_req = McpToolCallRequest {
        tool_name: "get_skill_directive".to_string(),
        arguments: json!({ "name": "bps-stat-methodology" }),
    };
    let get_res = mcp_registry.execute_tool(EnterpriseRole::Member, &get_req).await.expect("Execute get_skill_directive");
    assert!(!get_res.is_error);
    let get_json: serde_json::Value = serde_json::from_str(&get_res.content).expect("Valid JSON");
    assert_eq!(get_json["name"], "bps-stat-methodology");
    assert!(get_json["directives"].as_str().unwrap().contains("Margin of Error"));

    // 5. Test get_skill_directive with leading dollar sign ($spip-compliance)
    let get_dollar_req = McpToolCallRequest {
        tool_name: "get_skill_directive".to_string(),
        arguments: json!({ "name": "$spip-compliance" }),
    };
    let get_dollar_res = mcp_registry.execute_tool(EnterpriseRole::Member, &get_dollar_req).await.expect("Execute with $");
    assert!(!get_dollar_res.is_error);
    let dollar_json: serde_json::Value = serde_json::from_str(&get_dollar_res.content).expect("Valid JSON");
    assert_eq!(dollar_json["name"], "spip-compliance");
    assert!(dollar_json["directives"].as_str().unwrap().contains("Audit kontrol internal"));

    // 6. Test get_skill_directive for non-existent skill
    let not_found_req = McpToolCallRequest {
        tool_name: "get_skill_directive".to_string(),
        arguments: json!({ "name": "quantum-physics" }),
    };
    let not_found_res = mcp_registry.execute_tool(EnterpriseRole::Member, &not_found_req).await.expect("Execute not found");
    assert!(not_found_res.content.contains("not found"));

    // 7. Verify heal_or_infer_tool_name maps get_skill to get_skill_directive
    assert_eq!(
        heal_or_infer_tool_name("get_skill", &json!({"name": "bps-stat-methodology"})),
        "get_skill_directive"
    );
    assert_eq!(
        heal_or_infer_tool_name("", &json!({"skill_name": "data-analyst"})),
        "get_skill_directive"
    );
}
