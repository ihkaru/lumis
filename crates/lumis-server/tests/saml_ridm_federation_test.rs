use std::sync::Arc;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use lumis_core::traits::DefaultClaimTranslator;
use lumis_db::DatabaseFactory;
use lumis_server::guardrail::SimdGuardrailMatcher;
use lumis_server::routes::create_router;
use lumis_server::state::AppState;
use serde_json::Value;
use tower::ServiceExt;

async fn test_app() -> axum::Router {
    std::env::set_var("SAML_IDP_PROVIDER_NAME", "Enterprise_SAML");
    std::env::set_var("SAML_TENANT", "default");
    std::env::set_var("SAML_IDP_ENTITY_ID", "https://sso.enterprise.internal");
    std::env::set_var("SAML_SP_ENTITY_ID", "http://localhost:8080/saml/metadata");
    std::env::set_var("SAML_SP_ACS_URL", "http://localhost:8080/api/auth/saml/acs");
    std::env::set_var("SAML_SSO_URL", "https://sso.enterprise.internal/saml/sso");
    std::env::set_var("SAML_METADATA_URL", "https://sso.enterprise.internal/saml/metadata");

    let driver = DatabaseFactory::create_driver("sqlite::memory:")
        .await
        .expect("Initialize SQLite in-memory driver");
    driver.run_migrations().await.expect("Run migrations");

    let guardrail = Arc::new(
        SimdGuardrailMatcher::new(vec!["sk-ant-".to_string(), "DROP TABLE".to_string()])
            .expect("Guardrail init"),
    );
    let claim_translator = Arc::new(DefaultClaimTranslator::new());
    let state = AppState::new(
        driver,
        guardrail,
        claim_translator,
        "x-forwarded-user".to_string(),
    );
    create_router(state)
}

#[tokio::test]
async fn test_saml_config_endpoint() {
    let app = test_app().await;

    let req = Request::builder()
        .uri("/api/auth/saml/config")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let val: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(val["provider"], "Enterprise_SAML");
    assert_eq!(val["tenant"], "default");
    assert_eq!(val["idp_entity_id"], "https://sso.enterprise.internal");
}

#[tokio::test]
async fn test_saml_metadata_endpoint() {
    let app = test_app().await;

    let req = Request::builder()
        .uri("/api/auth/saml/metadata")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/samlmetadata+xml"
    );

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let xml = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(xml.contains("<md:EntityDescriptor"));
    assert!(xml.contains("http://localhost:8080/saml/metadata"));
    assert!(xml.contains("http://localhost:8080/api/auth/saml/acs"));
}

#[tokio::test]
async fn test_saml_sso_redirect_and_json() {
    let app = test_app().await;

    // 1. JSON mode
    let req_json = Request::builder()
        .uri("/api/auth/sso?format=json")
        .body(Body::empty())
        .unwrap();

    let resp_json = app.clone().oneshot(req_json).await.unwrap();
    assert_eq!(resp_json.status(), StatusCode::OK);

    let bytes = to_bytes(resp_json.into_body(), usize::MAX).await.unwrap();
    let val: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(val["redirect_url"].as_str().unwrap().starts_with("https://sso.enterprise.internal/saml/sso?SAMLRequest="));

    // 2. HTTP 302 Redirect mode
    let req_red = Request::builder()
        .uri("/api/auth/sso")
        .body(Body::empty())
        .unwrap();

    let resp_red = app.oneshot(req_red).await.unwrap();
    assert_eq!(resp_red.status(), StatusCode::FOUND);
    let loc = resp_red.headers().get(header::LOCATION).unwrap().to_str().unwrap();
    assert!(loc.starts_with("https://sso.enterprise.internal/saml/sso?SAMLRequest="));
}

#[tokio::test]
async fn test_saml_acs_consumption_and_session_cookie() {
    let app = test_app().await;

    let saml_xml = r#"
    <samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" ID="_resp_1">
        <saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_asst_1">
            <saml:Subject>
                <saml:NameID>pegawai.bps@bps.go.id</saml:NameID>
            </saml:Subject>
            <saml:AttributeStatement>
                <saml:Attribute Name="Role">
                    <saml:AttributeValue>developer,power_user</saml:AttributeValue>
                </saml:Attribute>
                <saml:Attribute Name="Department">
                    <saml:AttributeValue>BPS Mempawah</saml:AttributeValue>
                </saml:Attribute>
            </saml:AttributeStatement>
        </saml:Assertion>
    </samlp:Response>
    "#;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/saml/acs")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json")
        .body(Body::from(serde_json::to_vec(&serde_json::json!({
            "saml_response": saml_xml
        })).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::OK {
        panic!("ACS failed with status {}: {}", status, String::from_utf8_lossy(&bytes));
    }
    let val: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(val["status"], "authenticated");
    assert_eq!(val["user"]["username"], "pegawai.bps@bps.go.id");
}
