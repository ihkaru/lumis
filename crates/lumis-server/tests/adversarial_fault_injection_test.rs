#![deny(unsafe_code)]

use std::sync::Arc;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{
    ChatChunk, ChatCompletionRequest, ChatMessagePayload, EnterpriseRole, ProviderHealth,
    ProviderHealthStatus, User,
};
use lumis_core::traits::{DefaultClaimTranslator, LlmProviderDriver};
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::llm::ResilientLlmRouter;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use serde_json::Value;
use tokio::sync::mpsc;
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
async fn test_sentinel_round_trip_invariant_cannot_be_faked() {
    let (app, state) = test_app().await;

    // 1. Generate unique unguessable sentinel nonces
    let sentinel_user_id = format!("user-sentinel-{}", Uuid::new_v4());
    let sentinel_username = format!("pegawai_bps_{}", Uuid::new_v4());
    let sentinel_email = format!("{}@bps.go.id", sentinel_username);
    let sentinel_doc_token = format!("DIPA-SECRET-NONCE-{}", Uuid::new_v4());

    // 2. Sentinel Invariant 1: Physical User Persistence Round-Trip
    let user = User {
        id: sentinel_user_id.clone(),
        username: sentinel_username.clone(),
        email: Some(sentinel_email.clone()),
        role: EnterpriseRole::Member,
        tenant_id: Some("satker_3100".to_string()),
        preferences: serde_json::json!({ "theme": "system" }),
        is_active: true,
        password_hash: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    state.repo.upsert_user(&user).await.expect("Insert sentinel user");

    // Retrieve via HTTP API endpoint (/api/v1/auth/me) with ingress header
    let req = Request::builder()
        .uri("/api/auth/me")
        .header("x-forwarded-user", &sentinel_username)
        .header("x-forwarded-groups", "pegawai")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the exact unguessable sentinel username matches DB state
    assert_eq!(json["user"]["username"], sentinel_username);
    assert_eq!(json["role"], "member");

    // 3. Sentinel Invariant 2: Knowledge Ingestion Round-Trip
    let ingest_req = Request::builder()
        .method("POST")
        .uri("/api/knowledge/documents")
        .header("x-forwarded-user", &sentinel_username)
        .header("x-forwarded-groups", "pegawai")
        .header("x-tenant-id", "satker_3100")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "title": "Dokumen Uji Sentinel",
                "content": format!("Data statistik rahasia dengan kode validasi {}", sentinel_doc_token),
                "content_type": "text/plain",
                "min_role": "member",
            })
            .to_string(),
        ))
        .unwrap();

    let ingest_res = app.clone().oneshot(ingest_req).await.unwrap();
    assert_eq!(ingest_res.status(), StatusCode::CREATED);

    // Query with exact sentinel token
    let query_req = Request::builder()
        .method("POST")
        .uri("/api/knowledge/query")
        .header("x-forwarded-user", &sentinel_username)
        .header("x-forwarded-groups", "pegawai")
        .header("x-tenant-id", "satker_3100")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "query": format!("kode validasi {}", sentinel_doc_token),
                "top_k": 3,
            })
            .to_string(),
        ))
        .unwrap();

    let query_res = app.clone().oneshot(query_req).await.unwrap();
    assert_eq!(query_res.status(), StatusCode::OK);
    let q_body = to_bytes(query_res.into_body(), usize::MAX).await.unwrap();
    let results: Vec<Value> = serde_json::from_slice(&q_body).unwrap();

    // The unique sentinel token MUST be physically returned
    assert_eq!(results.len(), 1);
    assert!(
        results[0]["content"]
            .as_str()
            .unwrap()
            .contains(&sentinel_doc_token),
        "Sentinel token must be retrieved from physical store"
    );
}

// Simulated Failing LLM Driver for Fault Injection
struct OutageSimulatedDriver {
    name: String,
}

#[async_trait::async_trait]
impl LlmProviderDriver for OutageSimulatedDriver {
    fn provider_name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        Ok(ProviderHealth {
            provider_name: self.name.clone(),
            status: ProviderHealthStatus::Offline("Simulated 500 Outage".to_string()),
            latency_ms: 100,
        })
    }

    async fn complete_chat(
        &self,
        _request: &ChatCompletionRequest,
    ) -> Result<mpsc::Receiver<Result<ChatChunk>>> {
        Err(LumisError::Internal(
            "HTTP 500 Downstream LLM cluster outage simulated".to_string(),
        ))
    }
}

#[tokio::test]
async fn test_fault_injection_upstream_outage_fails_explicitly() {
    // 1. Construct resilient router with two drivers that both experience downstream outages
    let primary_faulty = Arc::new(OutageSimulatedDriver {
        name: "faulty_primary".to_string(),
    });
    let backup_faulty = Arc::new(OutageSimulatedDriver {
        name: "faulty_backup".to_string(),
    });

    let router = ResilientLlmRouter::new(
        primary_faulty,
        vec![backup_faulty],
        None,
    );

    let test_request = ChatCompletionRequest::new(
        "qwen2.5-coder",
        vec![ChatMessagePayload::new("user", "Tuliskan kode verifikasi integritas")],
    );

    // Attempt: Primary fails -> switches to backup -> backup fails
    let result = router.complete_chat(&test_request).await;

    // Invariant: The system MUST NOT return a fake dummy response with 200 OK.
    // It MUST return an explicit error.
    assert!(result.is_err(), "Must reject with explicit error upon outage");
    let err_str = result.err().unwrap().to_string();
    assert!(
        err_str.contains("outage") || err_str.contains("failed") || err_str.contains("unavailable"),
        "Error message must transparently report downstream failure: {}",
        err_str
    );
}
