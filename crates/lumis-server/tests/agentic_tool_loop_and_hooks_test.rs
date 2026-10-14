#![deny(unsafe_code)]

use std::sync::Arc;
use lumis_core::models::{EnterpriseRole, McpToolDefinition, User};
use lumis_server::audit::InMemoryAuditLogger;
use lumis_server::mcp::hooks::DlpScrubbingHook;
use lumis_server::mcp::{EnterpriseHookPipeline, McpExecutionHook};
use lumis_server::routes::chat::assembly::format_mcp_tools_for_openai;
use serde_json::json;

fn create_test_user() -> User {
    User {
        id: "usr_test_1".to_string(),
        username: "budi_statistisi".to_string(),
        email: Some("budi@bps.go.id".to_string()),
        role: EnterpriseRole::Member,
        tenant_id: Some("bps-kalbar".to_string()),
        preferences: json!({}),
        is_active: true,
        password_hash: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn test_dlp_scrubbing_hook_redacts_credentials_and_pii() {
    let hook = DlpScrubbingHook::new();
    let user = create_test_user();

    let sensitive_args = json!({
        "api_key": "sk-1234567890123456789012345",
        "nik": "6104123456780001",
        "credit_card": "4111 2222 3333 4444",
        "jwt": "eyJhGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozGzETcQ2",
        "safe_query": "cuaca pontianak hari ini"
    });

    let scrubbed = hook.before_call(&user, "test_tool", &sensitive_args).await.expect("DLP scrub succeeds");

    assert_eq!(scrubbed["api_key"], "[REDACTED_API_KEY]");
    assert_eq!(scrubbed["nik"], "[REDACTED_ID_NUMBER]");
    assert_eq!(scrubbed["credit_card"], "[REDACTED_CARD]");
    assert_eq!(scrubbed["jwt"], "[REDACTED_JWT]");
    assert_eq!(scrubbed["safe_query"], "cuaca pontianak hari ini");
}

#[tokio::test]
async fn test_enterprise_hook_pipeline_execution_order() {
    let audit_logger = Arc::new(InMemoryAuditLogger::default());
    let pipeline = EnterpriseHookPipeline::with_defaults(audit_logger);
    let user = create_test_user();

    let raw_args = json!({
        "token": "sk-abcdefghijklmnopqrstuvwxyz123",
        "search_term": "laporan inflasi"
    });

    let before_result = pipeline.execute_before(&user, "search_docs", &raw_args).await.expect("Pipeline before succeeds");
    assert_eq!(before_result["token"], "[REDACTED_API_KEY]");
    assert_eq!(before_result["search_term"], "laporan inflasi");

    let raw_output = json!({
        "status": "ok",
        "customer_nik": "6104123456780001"
    });

    let after_result = pipeline.execute_after(&user, "search_docs", &raw_output).await.expect("Pipeline after succeeds");
    assert_eq!(after_result["customer_nik"], "[REDACTED_ID_NUMBER]");
}

#[test]
fn test_format_mcp_tools_for_openai_schema() {
    let tools = vec![
        McpToolDefinition {
            name: "web_search".to_string(),
            description: "Search the web for up-to-date information".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
            min_role: EnterpriseRole::Member,
        },
        McpToolDefinition {
            name: "query_db".to_string(),
            description: "Query analytical database".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "sql": { "type": "string" }
                }
            }),
            min_role: EnterpriseRole::PowerUser,
        },
    ];

    let openai_tools = format_mcp_tools_for_openai(&tools);
    assert_eq!(openai_tools.len(), 2);
    assert_eq!(openai_tools[0]["type"], "function");
    assert_eq!(openai_tools[0]["function"]["name"], "web_search");
    assert_eq!(openai_tools[0]["function"]["parameters"]["properties"]["query"]["type"], "string");
    assert_eq!(openai_tools[1]["function"]["name"], "query_db");
}

#[test]
fn test_format_mcp_tools_sanitizes_slashes_and_colons_for_openai_models() {
    let complex_tools = vec![
        McpToolDefinition {
            name: "plan_task_create/update/delete/list".to_string(),
            description: "Task manager".to_string(),
            input_schema: json!({}),
            min_role: EnterpriseRole::Member,
        },
        McpToolDefinition {
            name: "propylon::execute_code".to_string(),
            description: "Hermetic sandbox".to_string(),
            input_schema: json!({ "properties": { "code": { "type": "string" } } }),
            min_role: EnterpriseRole::Member,
        },
    ];

    let formatted = format_mcp_tools_for_openai(&complex_tools);
    assert_eq!(formatted[0]["function"]["name"], "plan_task_create_update_delete_list");
    assert_eq!(formatted[0]["function"]["parameters"]["type"], "object");
    assert_eq!(formatted[1]["function"]["name"], "propylon__execute_code");
    assert_eq!(formatted[1]["function"]["parameters"]["type"], "object");
}

#[test]
fn test_heal_or_infer_tool_name_and_text_parsing() {
    use lumis_server::routes::chat::executor_helpers::{heal_or_infer_tool_name, parse_text_tool_calls};

    // 1. Raw name present
    assert_eq!(heal_or_infer_tool_name("execute_code", &json!({})), "execute_code");

    // 2. Empty raw name healed by argument inspection
    let code_args = json!({ "code": "print(123)", "language": "python" });
    assert_eq!(heal_or_infer_tool_name("", &code_args), "execute_code");

    let query_args = json!({ "query": "inflasi pontianak" });
    assert_eq!(heal_or_infer_tool_name("", &query_args), "gateway_search_tools");

    // 3. Parse flat text tool call without explicit name
    let text_with_tool = "Here is the code:\n<tool_call>\n{\"code\": \"print(5 * 3)\", \"language\": \"python\"}\n</tool_call>";
    let parsed = parse_text_tool_calls(text_with_tool);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].1, "execute_code");
    assert_eq!(parsed[0].2["code"], "print(5 * 3)");
}
