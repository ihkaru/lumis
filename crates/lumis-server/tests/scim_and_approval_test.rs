use std::sync::Arc;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use lumis_core::models::scim::{ScimEmail, ScimGroup, ScimMemberRef, ScimUser, SCIM_ERROR_SCHEMA, SCIM_USER_SCHEMA};
use lumis_core::models::EnterpriseRole;
use lumis_core::traits::DefaultClaimTranslator;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use tower::ServiceExt;

async fn test_app() -> (axum::Router, AppState) {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Initialize SQLite in-memory driver");
    driver.run_migrations().await.expect("Run migrations");

    let guardrail = Arc::new(
        SimdGuardrailMatcher::new(vec!["sk-test-secret".to_string()])
            .expect("Guardrail init"),
    );
    let claim_translator = Arc::new(DefaultClaimTranslator::new());
    let state = AppState::new(
        driver,
        guardrail,
        claim_translator,
        "x-forwarded-user".to_string(),
    );
    let router = create_router(state.clone());
    (router, state)
}

#[tokio::test]
async fn test_scim_user_and_group_lifecycle_rfc7644() {
    let (app, _) = test_app().await;

    // 1. Create SCIM User (POST /scim/v2/Users)
    let new_user = ScimUser {
        schemas: vec![SCIM_USER_SCHEMA.to_string()],
        id: "usr-emp-101".to_string(),
        user_name: "john.doe".to_string(),
        display_name: Some("John Doe".to_string()),
        emails: vec![ScimEmail {
            value: "john.doe@enterprise.com".to_string(),
            primary: true,
        }],
        active: true,
        roles: vec!["PowerUser".to_string()],
    };

    let create_req = Request::builder()
        .method("POST")
        .uri("/scim/v2/Users")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&new_user).unwrap()))
        .unwrap();

    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    // 2. Query SCIM Users (GET /scim/v2/Users)
    let list_req = Request::builder()
        .method("GET")
        .uri("/scim/v2/Users")
        .body(Body::empty())
        .unwrap();

    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_bytes = to_bytes(list_resp.into_body(), usize::MAX).await.unwrap();
    let list_json: serde_json::Value = serde_json::from_slice(&list_bytes).unwrap();
    assert_eq!(list_json["totalResults"], 1);
    assert_eq!(list_json["Resources"][0]["userName"], "john.doe");

    // 3. De-provision / Deactivate user (PATCH /scim/v2/Users/{id})
    let patch_req = Request::builder()
        .method("PATCH")
        .uri("/scim/v2/Users/usr-emp-101")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::json!({ "active": false }).to_string()))
        .unwrap();

    let patch_resp = app.clone().oneshot(patch_req).await.unwrap();
    assert_eq!(patch_resp.status(), StatusCode::NO_CONTENT);

    // Verify user is now deactivated
    let get_req = Request::builder()
        .method("GET")
        .uri("/scim/v2/Users/usr-emp-101")
        .body(Body::empty())
        .unwrap();

    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_bytes = to_bytes(get_resp.into_body(), usize::MAX).await.unwrap();
    let user_json: serde_json::Value = serde_json::from_slice(&get_bytes).unwrap();
    assert_eq!(user_json["active"], false);

    // 4. Create SCIM Group (POST /scim/v2/Groups)
    let group = ScimGroup {
        schemas: vec![],
        id: "grp-engineers".to_string(),
        display_name: "Data_Engineers".to_string(),
        members: vec![ScimMemberRef {
            value: "usr-emp-101".to_string(),
            display: Some("John Doe".to_string()),
        }],
    };

    let group_req = Request::builder()
        .method("POST")
        .uri("/scim/v2/Groups")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&group).unwrap()))
        .unwrap();

    let group_resp = app.clone().oneshot(group_req).await.unwrap();
    assert_eq!(group_resp.status(), StatusCode::CREATED);

    // 5. Delete SCIM User (DELETE /scim/v2/Users/{id})
    let del_req = Request::builder()
        .method("DELETE")
        .uri("/scim/v2/Users/usr-emp-101")
        .body(Body::empty())
        .unwrap();

    let del_resp = app.clone().oneshot(del_req).await.unwrap();
    assert_eq!(del_resp.status(), StatusCode::NO_CONTENT);

    // 6. Verify subsequent fetch returns 404 with standard SCIM Error schema
    let not_found_req = Request::builder()
        .method("GET")
        .uri("/scim/v2/Users/usr-emp-101")
        .body(Body::empty())
        .unwrap();

    let not_found_resp = app.oneshot(not_found_req).await.unwrap();
    assert_eq!(not_found_resp.status(), StatusCode::NOT_FOUND);
    let err_bytes = to_bytes(not_found_resp.into_body(), usize::MAX).await.unwrap();
    let err_json: serde_json::Value = serde_json::from_slice(&err_bytes).unwrap();
    assert_eq!(err_json["schemas"][0], SCIM_ERROR_SCHEMA);
}

#[tokio::test]
async fn test_human_in_the_loop_tool_approval_workflow() {
    let (app, state) = test_app().await;

    // 1. Model / Agent requests approval for destructive MCP tool
    let approval_req = state
        .approval_manager
        .request_approval(
            "destructive_drop_table",
            &serde_json::json!({ "table_name": "customer_ledger" }),
            "analyst_bob",
            EnterpriseRole::Member,
        )
        .await
        .unwrap();

    let token = approval_req.token.clone();
    assert!(approval_req.is_pending());

    // 2. Member user attempts to view pending approvals -> 403 Forbidden
    let unpriv_req = Request::builder()
        .method("GET")
        .uri("/api/mcp/approvals/pending")
        .header("x-forwarded-user", "analyst_bob")
        .header("x-forwarded-groups", "member")
        .body(Body::empty())
        .unwrap();

    let unpriv_resp = app.clone().oneshot(unpriv_req).await.unwrap();
    assert_eq!(unpriv_resp.status(), StatusCode::FORBIDDEN);

    // 3. TenantAdmin views pending approvals -> 200 OK with list containing our token
    let admin_list_req = Request::builder()
        .method("GET")
        .uri("/api/mcp/approvals/pending")
        .header("x-forwarded-user", "admin_carol")
        .header("x-forwarded-groups", "tenant_admin")
        .body(Body::empty())
        .unwrap();

    let admin_list_resp = app.clone().oneshot(admin_list_req).await.unwrap();
    assert_eq!(admin_list_resp.status(), StatusCode::OK);
    let list_bytes = to_bytes(admin_list_resp.into_body(), usize::MAX).await.unwrap();
    let list_json: serde_json::Value = serde_json::from_slice(&list_bytes).unwrap();
    assert_eq!(list_json.as_array().unwrap().len(), 1);
    assert_eq!(list_json[0]["token"], token);

    // 4. TenantAdmin approves request (POST /api/mcp/approvals/{token}/approve)
    let approve_req = Request::builder()
        .method("POST")
        .uri(format!("/api/mcp/approvals/{}/approve", token))
        .header("x-forwarded-user", "admin_carol")
        .header("x-forwarded-groups", "tenant_admin")
        .body(Body::empty())
        .unwrap();

    let approve_resp = app.clone().oneshot(approve_req).await.unwrap();
    assert_eq!(approve_resp.status(), StatusCode::OK);
    let appr_bytes = to_bytes(approve_resp.into_body(), usize::MAX).await.unwrap();
    let appr_json: serde_json::Value = serde_json::from_slice(&appr_bytes).unwrap();
    assert_eq!(appr_json["status"]["approved"]["approved_by"], "admin_carol");

    // 5. Verify SIEM audit trail contains approval record
    let events = state
        .audit_logger
        .query_recent_events("default", EnterpriseRole::TenantAdmin, 50)
        .await
        .unwrap();
    let approval_event = events
        .iter()
        .find(|e| e.action == "tool_approval_granted")
        .expect("SIEM audit log must contain tool_approval_granted event");
    assert_eq!(approval_event.actor_id, "admin_carol");
    assert_eq!(approval_event.resource_id, Some(token.clone()));
}
