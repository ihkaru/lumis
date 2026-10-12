#![deny(unsafe_code)]

use std::sync::Arc;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use chrono::Utc;
use lumis_core::models::{AuditEventStatus, AuditLogEvent, EnterpriseRole};
use lumis_core::traits::DefaultClaimTranslator;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

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
async fn test_siem_log_formatters_ecs_cef_syslog() {
    let event = AuditLogEvent {
        event_id: format!("evt-{}", Uuid::new_v4()),
        timestamp: Utc::now(),
        tenant_id: "satker_3100".to_string(),
        actor_id: "staff_operator".to_string(),
        actor_role: EnterpriseRole::Member,
        action: "mcp.tool_called".to_string(),
        resource_id: Some("system_status".to_string()),
        status: AuditEventStatus::Success,
        ip_address: Some("192.168.1.100".to_string()),
        user_agent: Some("LumisClient/1.0".to_string()),
        details: serde_json::json!({ "latency_ms": 12 }),
    };

    // 1. Elastic Common Schema (ECS v8.x) JSON verification
    let ecs = event.to_ecs_json();
    assert_eq!(ecs["ecs.version"], "8.11.0");
    assert_eq!(ecs["event"]["action"], "mcp.tool_called");
    assert_eq!(ecs["event"]["outcome"], "success");
    assert_eq!(ecs["user"]["id"], "staff_operator");
    assert_eq!(ecs["organization"]["id"], "satker_3100");
    assert_eq!(ecs["client"]["ip"], "192.168.1.100");

    // 2. ArcSight Common Event Format (CEF:0) verification
    let cef = event.to_cef_string();
    assert!(cef.starts_with("CEF:0|Lumis|LumisPlatform|1.0|mcp.tool_called|"));
    assert!(cef.contains("suser=staff_operator"));
    assert!(cef.contains("cs1=satker_3100"));
    assert!(cef.contains("outcome=success"));

    // 3. Syslog RFC 5424 verification
    let syslog = event.to_syslog_rfc5424();
    assert!(syslog.starts_with("<134>1"));
    assert!(syslog.contains("lumis-server lumis-audit"));
    assert!(syslog.contains("tenantId=\"satker_3100\""));
    assert!(syslog.contains("actor=\"staff_operator\""));
}

#[tokio::test]
async fn test_role_gated_siem_audit_log_access() {
    let (app, state) = test_app().await;

    // Log a sensitive administrative event into SIEM
    let audit_event = AuditLogEvent {
        event_id: format!("evt-{}", Uuid::new_v4()),
        timestamp: Utc::now(),
        tenant_id: "satker_3100".to_string(),
        actor_id: "ciso_admin".to_string(),
        actor_role: EnterpriseRole::TenantAdmin,
        action: "guardrail.dlp_violation".to_string(),
        resource_id: Some("pii_filter".to_string()),
        status: AuditEventStatus::Denied,
        ip_address: Some("10.0.0.5".to_string()),
        user_agent: Some("SecOps/2.0".to_string()),
        details: serde_json::json!({ "pattern": "NIK_REDACTED" }),
    };
    state.audit_logger.log_event(&audit_event).await.unwrap();

    // 1. Member tries to view SIEM audit logs -> MUST RETURN 403 FORBIDDEN
    let member_req = Request::builder()
        .uri("/api/admin/audit/events")
        .header("x-forwarded-user", "regular_staff")
        .header("x-forwarded-groups", "pegawai") // Maps to Member
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(member_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 2. TenantAdmin views SIEM audit logs -> SUCCEEDS 200 OK
    let admin_req = Request::builder()
        .uri("/api/admin/audit/events?format=ecs")
        .header("x-forwarded-user", "satker_lead")
        .header("x-forwarded-groups", "manager") // Maps to TenantAdmin
        .header("x-tenant-id", "satker_3100")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(admin_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let events: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert!(!events.is_empty());
    assert_eq!(events[0]["ecs.version"], "8.11.0");
    assert_eq!(events[0]["event"]["action"], "guardrail.dlp_violation");
}

#[tokio::test]
async fn test_soc_emergency_killswitch_lockdown_submillisecond() {
    let (app, _) = test_app().await;

    // 1. Member tries to toggle killswitch -> MUST RETURN 403 FORBIDDEN
    let member_kill_req = Request::builder()
        .method("POST")
        .uri("/api/admin/emergency/killswitch")
        .header("x-forwarded-user", "regular_staff")
        .header("x-forwarded-groups", "pegawai")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "active": true,
                "reason": "Unauthorized attempt",
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(member_kill_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 2. TenantAdmin activates emergency killswitch
    let admin_kill_req = Request::builder()
        .method("POST")
        .uri("/api/admin/emergency/killswitch")
        .header("x-forwarded-user", "ciso_officer")
        .header("x-forwarded-groups", "ciso") // Maps to SuperAdmin
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "active": true,
                "reason": "National Cyber Threat Lockdown",
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(admin_kill_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let status_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(status_json["active"], true);
    assert_eq!(status_json["reason"], "National Cyber Threat Lockdown");

    // 3. User attempts to send chat message during lockdown -> MUST IMMEDIATELY RETURN 503 SERVICE UNAVAILABLE
    let chat_req = Request::builder()
        .method("POST")
        .uri("/api/sessions/sess-lockdown-test/messages")
        .header("x-forwarded-user", "regular_staff")
        .header("x-forwarded-groups", "pegawai")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "content": "Halo, bisakah saya bertanya?",
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(chat_req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "All requests must be blocked sub-millisecond when killswitch is active"
    );
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let err_json: Value = serde_json::from_slice(&body).unwrap();
    assert!(err_json["error"].as_str().unwrap().contains("National Cyber Threat Lockdown"));

    // 4. Deactivate emergency killswitch
    let deactivate_req = Request::builder()
        .method("POST")
        .uri("/api/admin/emergency/killswitch")
        .header("x-forwarded-user", "ciso_officer")
        .header("x-forwarded-groups", "ciso")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "active": false,
                "reason": "Threat neutralized",
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(deactivate_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
