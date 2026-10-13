#![deny(unsafe_code)]

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Helper function to perform standard RFC 3986 percent-encoding without external crates.
pub fn percent_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

/// PKCE (RFC 7636) code verifier and S256 challenge pair for OAuth 2.1.
#[derive(Debug, Clone)]
pub struct PkcePair {
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: &'static str,
}

/// Generates a cryptographically strong PKCE pair using SHA-256 (S256).
pub fn generate_pkce_pair() -> PkcePair {
    let rand_a = *Uuid::new_v4().as_bytes();
    let rand_b = *Uuid::new_v4().as_bytes();
    let mut verifier_bytes = [0u8; 32];
    verifier_bytes[0..16].copy_from_slice(&rand_a);
    verifier_bytes[16..32].copy_from_slice(&rand_b);

    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    let code_challenge = URL_SAFE_NO_PAD.encode(hash);

    PkcePair {
        code_verifier,
        code_challenge,
        code_challenge_method: "S256",
    }
}

/// Constructs a standardized OAuth 2.1 Authorization URL for MCP Servers.
/// Supports RFC 8707 Resource Indicators and mandatory PKCE.
pub fn build_mcp_authorization_url(
    authorization_endpoint: &str,
    client_id: &str,
    redirect_uri: &str,
    code_challenge: &str,
    scopes: &[String],
    state: &str,
    resource_indicator: Option<&str>,
) -> String {
    let base = authorization_endpoint.trim_end_matches('?');
    let scope_str = scopes.join(" ");

    let mut query_params = vec![
        format!("response_type=code"),
        format!("client_id={}", percent_encode(client_id)),
        format!("redirect_uri={}", percent_encode(redirect_uri)),
        format!("code_challenge={}", percent_encode(code_challenge)),
        format!("code_challenge_method=S256"),
        format!("state={}", percent_encode(state)),
    ];

    if !scope_str.is_empty() {
        query_params.push(format!("scope={}", percent_encode(&scope_str)));
    }

    if let Some(res) = resource_indicator {
        if !res.is_empty() {
            query_params.push(format!("resource={}", percent_encode(res)));
        }
    }

    let separator = if base.contains('?') { "&" } else { "?" };
    format!("{}{}{}", base, separator, query_params.join("&"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation_and_s256_invariants() {
        let pkce = generate_pkce_pair();
        assert!(!pkce.code_verifier.is_empty());
        assert!(!pkce.code_challenge.is_empty());
        assert_eq!(pkce.code_challenge_method, "S256");

        // Verify SHA-256 S256 derivation mathematically
        let mut hasher = Sha256::new();
        hasher.update(pkce.code_verifier.as_bytes());
        let expected_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());
        assert_eq!(pkce.code_challenge, expected_challenge);
    }

    #[test]
    fn test_build_mcp_authorization_url_with_resource_indicator() {
        let url = build_mcp_authorization_url(
            "https://auth.enterprise.com/oauth/authorize",
            "lumis-client-01",
            "https://lumis.internal/api/mcp/oauth/callback",
            "dummy_challenge_s256",
            &["read".to_string(), "tools:invoke".to_string()],
            "state_token_123",
            Some("https://mcp.enterprise.com/ledger"),
        );

        assert!(url.starts_with("https://auth.enterprise.com/oauth/authorize?"));
        assert!(url.contains("client_id=lumis-client-01"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("code_challenge=dummy_challenge_s256"));
        assert!(url.contains("state=state_token_123"));
        assert!(url.contains("resource=https%3A%2F%2Fmcp.enterprise.com%2Fledger"));
    }
}
