use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use lumis_core::models::EnterpriseRole;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Write;

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct SamlConfigResponse {
    pub provider: String,
    pub tenant: String,
    pub idp_entity_id: String,
    pub idp_sso_url: String,
    pub idp_metadata_url: String,
    pub sp_entity_id: String,
    pub sp_acs_url: String,
}

#[derive(Debug, Deserialize)]
pub struct SsoRedirectQuery {
    pub format: Option<String>,
    pub redirect: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AcsFormPayload {
    #[serde(rename = "SAMLResponse")]
    pub saml_response: Option<String>,
    #[serde(rename = "RelayState")]
    pub relay_state: Option<String>,
}

fn get_idp_sso_url() -> String {
    env::var("SAML_SSO_URL")
        .or_else(|_| env::var("RIDM_BPS_SAML_SSO_URL"))
        .unwrap_or_else(|_| "https://auth.dvlpid.my.id/t/bps/saml/sso".to_string())
}

fn get_idp_metadata_url() -> String {
    env::var("SAML_METADATA_URL")
        .or_else(|_| env::var("RIDM_BPS_SAML_METADATA_URL"))
        .unwrap_or_else(|_| "https://auth.dvlpid.my.id/t/bps/saml/metadata".to_string())
}

fn get_sp_entity_id() -> String {
    env::var("SAML_SP_ENTITY_ID")
        .or_else(|_| env::var("APP_URL").map(|u| format!("{}/saml/metadata", u.trim_end_matches('/'))))
        .unwrap_or_else(|_| "https://chat.dvlpid.my.id/saml/metadata".to_string())
}

fn get_sp_acs_url() -> String {
    env::var("SAML_SP_ACS_URL")
        .or_else(|_| env::var("APP_URL").map(|u| format!("{}/api/auth/saml/acs", u.trim_end_matches('/'))))
        .unwrap_or_else(|_| "https://chat.dvlpid.my.id/api/auth/saml/acs".to_string())
}

pub async fn saml_config() -> impl IntoResponse {
    let resp = SamlConfigResponse {
        provider: env::var("SAML_IDP_PROVIDER_NAME").unwrap_or_else(|_| "rIDM BPS SSO".to_string()),
        tenant: env::var("SAML_TENANT").unwrap_or_else(|_| "bps".to_string()),
        idp_entity_id: env::var("SAML_IDP_ENTITY_ID").unwrap_or_else(|_| "https://auth.dvlpid.my.id/t/bps".to_string()),
        idp_sso_url: get_idp_sso_url(),
        idp_metadata_url: get_idp_metadata_url(),
        sp_entity_id: get_sp_entity_id(),
        sp_acs_url: get_sp_acs_url(),
    };
    (StatusCode::OK, Json(resp))
}

pub async fn saml_metadata() -> impl IntoResponse {
    let sp_id = get_sp_entity_id();
    let acs = get_sp_acs_url();
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<md:EntityDescriptor xmlns:md=\"urn:oasis:names:tc:SAML:2.0:metadata\" entityID=\"{sp_id}\">\
<md:SPSSODescriptor protocolSupportEnumeration=\"urn:oasis:names:tc:SAML:2.0:protocol\" AuthnRequestsSigned=\"false\" WantAssertionsSigned=\"false\">\
<md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat>\
<md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified</md:NameIDFormat>\
<md:AssertionConsumerService Binding=\"urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST\" Location=\"{acs}\" index=\"0\" isDefault=\"true\"/>\
</md:SPSSODescriptor>\
</md:EntityDescriptor>"
    );

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/samlmetadata+xml")],
        xml,
    )
}

pub async fn saml_login(Query(query): Query<SsoRedirectQuery>) -> Response {
    let sso_url = get_idp_sso_url();
    let sp_id = get_sp_entity_id();
    let acs = get_sp_acs_url();
    let req_id = format!("_lumis_{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let authn_xml = format!(
        "<samlp:AuthnRequest xmlns:samlp=\"urn:oasis:names:tc:SAML:2.0:protocol\" \
ID=\"{req_id}\" Version=\"2.0\" IssueInstant=\"{now}\" \
Destination=\"{sso_url}\" AssertionConsumerServiceURL=\"{acs}\"> \
<saml:Issuer xmlns:saml=\"urn:oasis:names:tc:SAML:2.0:assertion\">{sp_id}</saml:Issuer> \
<samlp:NameIDPolicy Format=\"urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified\" AllowCreate=\"true\"/> \
</samlp:AuthnRequest>"
    );

    // Deflate and Base64 encode
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    let b64 = match encoder.write_all(authn_xml.as_bytes()).and_then(|_| encoder.finish()) {
        Ok(compressed) => BASE64_STANDARD.encode(&compressed),
        Err(_) => BASE64_STANDARD.encode(authn_xml.as_bytes()),
    };

    let encoded_saml = url_encode(&b64);
    let mut redirect_url = format!("{sso_url}?SAMLRequest={encoded_saml}");
    if let Some(r) = query.redirect {
        redirect_url.push_str(&format!("&RelayState={}", url_encode(&r)));
    }

    if query.format.as_deref() == Some("json") {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "sso_url": sso_url,
                "redirect_url": redirect_url,
                "saml_request": b64
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::FOUND,
            [(header::LOCATION, redirect_url)],
            (),
        )
            .into_response()
    }
}

pub async fn saml_acs(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let body_str = String::from_utf8_lossy(&body);
    let raw_payload = if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&body) {
        json_val.get("saml_response").and_then(|v| v.as_str()).map(str::to_string)
    } else {
        body_str.split('&').find_map(|pair| {
            let mut kv = pair.splitn(2, '=');
            match (kv.next(), kv.next()) {
                (Some("SAMLResponse"), Some(val)) => {
                    let decoded = url_decode(val);
                    if decoded.is_empty() { None } else { Some(decoded) }
                }
                _ => None,
            }
        })
    };

    let saml_xml = match raw_payload {
        Some(raw) => {
            if raw.trim().starts_with('<') {
                raw
            } else if let Ok(decoded) = BASE64_STANDARD.decode(raw.trim()) {
                String::from_utf8_lossy(&decoded).into_owned()
            } else {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid Base64 payload" }))).into_response();
            }
        }
        None => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing SAMLResponse parameter" }))).into_response();
        }
    };

    let name_id = extract_tag(&saml_xml, "NameID").or_else(|| extract_tag(&saml_xml, "nameid"));
    let nip = extract_attr_val(&saml_xml, "nip");
    let email = extract_attr_val(&saml_xml, "email").or_else(|| {
        if name_id.as_ref().is_some_and(|s| s.contains('@')) {
            name_id.clone()
        } else {
            None
        }
    });
    let preferred_username = extract_attr_val(&saml_xml, "panggilan")
        .or_else(|| extract_attr_val(&saml_xml, "name"))
        .or_else(|| extract_attr_val(&saml_xml, "username"))
        .or_else(|| name_id.clone())
        .or_else(|| nip.clone())
        .unwrap_or_else(|| "saml_user".to_string());

    let subject = nip.or(name_id).unwrap_or_else(|| preferred_username.clone());

    let role_raw = extract_attr_val(&saml_xml, "Role")
        .or_else(|| extract_attr_val(&saml_xml, "groups"))
        .unwrap_or_else(|| "member".to_string());

    let role = if role_raw.contains("admin") || role_raw.contains("superadmin") {
        EnterpriseRole::SuperAdmin
    } else if role_raw.contains("power_user") || role_raw.contains("developer") {
        EnterpriseRole::PowerUser
    } else {
        EnterpriseRole::Member
    };

    let user = match crate::routes::chat::helpers::ensure_federated_user(
        &state,
        "saml",
        &subject,
        &preferred_username,
        role,
        email,
    )
    .await
    {
        Ok(u) => u,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))).into_response(),
    };

    let cookie_val = format!("lumis_user={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000", user.username);
    let mut resp = if headers.get(header::ACCEPT).and_then(|v| v.to_str().ok()).unwrap_or("").contains("application/json") {
        (StatusCode::OK, Json(serde_json::json!({ "status": "authenticated", "user": user }))).into_response()
    } else {
        (StatusCode::FOUND, [(header::LOCATION, "/")], ()).into_response()
    };

    if let Ok(hdr) = HeaderValue::from_str(&cookie_val) {
        resp.headers_mut().insert(header::SET_COOKIE, hdr);
    }
    resp
}

fn extract_tag(xml: &str, tag_name: &str) -> Option<String> {
    let prefixes = ["saml:", "saml2:", "samlp:", ""];
    for prefix in prefixes {
        let full_tag = format!("<{prefix}{tag_name}");
        let close_tag = format!("</{prefix}{tag_name}>");

        let mut search_idx = 0;
        while let Some(start) = xml[search_idx..].find(&full_tag) {
            let tag_start = search_idx + start;
            let after_tag = tag_start + full_tag.len();
            if after_tag < xml.len() {
                let next_char = xml.as_bytes()[after_tag];
                if next_char == b'>' || next_char.is_ascii_whitespace() {
                    if let Some(gt_pos) = xml[after_tag..].find('>') {
                        let content_start = after_tag + gt_pos + 1;
                        if let Some(end) = xml[content_start..].find(&close_tag) {
                            return Some(xml[content_start..content_start + end].trim().to_string());
                        }
                    }
                }
            }
            search_idx = after_tag;
        }
    }
    None
}

fn extract_attr_val(xml: &str, attribute_name: &str) -> Option<String> {
    let pattern = format!("Name=\"{attribute_name}\"");
    if let Some(pos) = xml.find(&pattern) {
        let slice = &xml[pos..];
        let val_variants = ["<saml:AttributeValue", "<saml2:AttributeValue", "<AttributeValue"];
        for val_tag in val_variants {
            if let Some(start) = slice.find(val_tag) {
                if let Some(gt) = slice[start..].find('>') {
                    let content_start = start + gt + 1;
                    if let Some(end) = slice[content_start..].find("</") {
                        return Some(slice[content_start..content_start + end].trim().to_string());
                    }
                }
            }
        }
    }
    None
}

fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

fn url_decode(input: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                if let Ok(val) = u8::from_str_radix(&format!("{}{}", h1 as char, h2 as char), 16) {
                    bytes.push(val);
                    continue;
                }
            }
        } else if b == b'+' {
            bytes.push(b' ');
            continue;
        }
        bytes.push(b);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}
