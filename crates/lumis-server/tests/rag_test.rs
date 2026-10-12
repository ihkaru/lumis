use std::sync::Arc;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use lumis_core::traits::DefaultClaimTranslator;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::rag::HybridKnowledgeVault;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use tower::ServiceExt;

async fn test_app() -> axum::Router {
    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .unwrap();
    driver.run_migrations().await.unwrap();

    let guardrail = Arc::new(SimdGuardrailMatcher::new(vec!["sk-test".to_string()]).unwrap());
    let claim_translator = Arc::new(DefaultClaimTranslator::new());
    let quota_engine = Arc::new(lumis_server::finops::InMemoryQuotaEngine::new());
    let telemetry = Arc::new(lumis_server::finops::PrometheusTelemetryCollector::new());
    let knowledge_vault = Arc::new(HybridKnowledgeVault::new());
    let mcp_registry = Arc::new(lumis_server::mcp::InMemoryMcpRegistry::default());
    let audit_logger = Arc::new(lumis_server::audit::InMemoryAuditLogger::default());

    let state = AppState::with_options(
        driver,
        guardrail,
        claim_translator,
        quota_engine,
        telemetry,
        knowledge_vault,
        mcp_registry,
        audit_logger,
        "x-forwarded-user".to_string(),
    );

    create_router(state)
}

#[tokio::test]
async fn test_role_gated_hybrid_rag_isolation() {
    let app = test_app().await;

    // 1. Admin uploads confidential executive document (min_role: tenant_admin)
    let admin_doc_req = Request::builder()
        .method("POST")
        .uri("/api/knowledge/documents")
        .header("x-forwarded-user", "satker_lead")
        .header("x-forwarded-groups", "manager") // Maps to TenantAdmin
        .header("x-tenant-id", "provinsi_01")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "title": "Dokumen Rahasia Anggaran 2026",
                "content": "Alokasi anggaran rahasia pimpinan untuk satker provinsi adalah Rp 50 Miliar dengan kode sandi DIPA-998877.",
                "content_type": "text/markdown",
                "min_role": "tenant_admin",
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(admin_doc_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 2. Member uploads general staff handbook (min_role: member)
    let member_doc_req = Request::builder()
        .method("POST")
        .uri("/api/knowledge/documents")
        .header("x-forwarded-user", "staff_member")
        .header("x-forwarded-groups", "pegawai") // Maps to Member
        .header("x-tenant-id", "provinsi_01")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "title": "Panduan Jam Kerja dan Absensi",
                "content": "Seluruh pegawai wajib melakukan presensi kehadiran sebelum pukul 07.30 pagi setiap hari kerja.",
                "content_type": "text/markdown",
                "min_role": "member",
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(member_doc_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 3. Test Security Isolation: Guest user queries for "anggaran rahasia" -> MUST BE EMPTY (0 results)
    let guest_query_req = Request::builder()
        .method("POST")
        .uri("/api/knowledge/query")
        .header("x-forwarded-user", "guest_intern")
        .header("x-forwarded-groups", "tamu") // Maps to Guest (level 1)
        .header("x-tenant-id", "provinsi_01")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "query": "anggaran rahasia DIPA-998877 pimpinan",
                "top_k": 5,
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(guest_query_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(results.len(), 0, "Guest must not receive any confidential results");

    // 4. Test Security Isolation: Member queries for "anggaran rahasia" -> MUST STILL BE EMPTY
    let member_leak_test_req = Request::builder()
        .method("POST")
        .uri("/api/knowledge/query")
        .header("x-forwarded-user", "staff_member")
        .header("x-forwarded-groups", "pegawai") // Maps to Member (level 2)
        .header("x-tenant-id", "provinsi_01")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "query": "anggaran rahasia DIPA-998877 pimpinan",
                "top_k": 5,
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(member_leak_test_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(results.len(), 0, "Member must not see TenantAdmin documents");

    // 5. Test Authorized Access: Member queries for "presensi kehadiran jam kerja" -> MUST RETURN HANDBOOK
    let member_valid_req = Request::builder()
        .method("POST")
        .uri("/api/knowledge/query")
        .header("x-forwarded-user", "staff_member")
        .header("x-forwarded-groups", "pegawai")
        .header("x-tenant-id", "provinsi_01")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "query": "presensi kehadiran pegawai jam kerja",
                "top_k": 5,
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(member_valid_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(!results.is_empty(), "Member must receive handbook results");
    assert!(results[0]["content"].as_str().unwrap().contains("07.30 pagi"));

    // 6. Test Authorized Access: TenantAdmin queries for "anggaran rahasia" -> MUST RETURN EXECUTIVE DOC
    let admin_valid_req = Request::builder()
        .method("POST")
        .uri("/api/knowledge/query")
        .header("x-forwarded-user", "satker_lead")
        .header("x-forwarded-groups", "manager")
        .header("x-tenant-id", "provinsi_01")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "query": "anggaran rahasia DIPA-998877 pimpinan",
                "top_k": 5,
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(admin_valid_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(!results.is_empty());
    assert!(results[0]["content"].as_str().unwrap().contains("DIPA-998877"));
}
