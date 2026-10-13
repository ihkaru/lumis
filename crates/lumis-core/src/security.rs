#![deny(unsafe_code)]

use std::net::IpAddr;
use std::str::FromStr;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use crate::error::{LumisError, Result};

/// Validates an outbound MCP server endpoint URL to prevent Server-Side Request Forgery (SSRF).
/// Enforces strict RFC 1918, loopback, and Cloud Metadata (169.254.169.254) blocklist rules.
pub fn validate_mcp_endpoint_url(url_str: &str, allowlist: &[String]) -> Result<()> {
    let trimmed = url_str.trim();
    if trimmed.is_empty() {
        return Err(LumisError::Validation("Endpoint URL cannot be empty".to_string()));
    }

    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(LumisError::Validation("Endpoint URL must use http or https scheme".to_string()));
    }

    let url_without_scheme = if let Some(stripped) = trimmed.strip_prefix("https://") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("http://") {
        stripped
    } else {
        trimmed
    };

    let host_and_port = url_without_scheme.split('/').next().unwrap_or("");
    let host = host_and_port.split(':').next().unwrap_or("").trim();

    if host.is_empty() {
        return Err(LumisError::Validation("Endpoint URL host cannot be empty".to_string()));
    }

    // 1. Block known dangerous hostname strings
    let lower_host = host.to_lowercase();
    if lower_host == "localhost"
        || lower_host.ends_with(".localhost")
        || lower_host == "metadata.google.internal"
        || lower_host == "instance-data"
    {
        return Err(LumisError::Validation(format!(
            "Access to internal host '{}' is blocked by anti-SSRF policy",
            host
        )));
    }

    // 2. Parse IP address (IPv4 or IPv6) and block dangerous CIDRs
    if let Ok(ip) = IpAddr::from_str(host) {
        if ip.is_loopback() {
            return Err(LumisError::Validation(format!(
                "Loopback IP address '{}' is blocked by anti-SSRF policy",
                ip
            )));
        }

        match ip {
            IpAddr::V4(v4) => {
                let octets = v4.octets();
                // 10.0.0.0/8 (RFC 1918)
                if octets[0] == 10 {
                    return Err(LumisError::Validation("Private IP space 10.0.0.0/8 is blocked".to_string()));
                }
                // 172.16.0.0/12 (RFC 1918)
                if octets[0] == 172 && (16..=31).contains(&octets[1]) {
                    return Err(LumisError::Validation("Private IP space 172.16.0.0/12 is blocked".to_string()));
                }
                // 192.168.0.0/16 (RFC 1918)
                if octets[0] == 192 && octets[1] == 168 {
                    return Err(LumisError::Validation("Private IP space 192.168.0.0/16 is blocked".to_string()));
                }
                // 169.254.0.0/16 (Link-local & AWS/GCP/Azure Cloud Metadata)
                if octets[0] == 169 && octets[1] == 254 {
                    return Err(LumisError::Validation(
                        "Cloud Metadata / Link-local IP 169.254.0.0/16 is blocked".to_string(),
                    ));
                }
                // 0.0.0.0 or 255.255.255.255
                if octets[0] == 0 || octets[0] == 255 {
                    return Err(LumisError::Validation("Broadcast/unspecified IP address is blocked".to_string()));
                }
            }
            IpAddr::V6(v6) => {
                if v6.is_unspecified() {
                    return Err(LumisError::Validation("Unspecified IPv6 address is blocked".to_string()));
                }
            }
        }
    }

    // 3. Domain allowlist enforcement (if non-empty)
    if !allowlist.is_empty() {
        let is_allowed = allowlist.iter().any(|allowed| {
            let allowed_clean = allowed.trim().to_lowercase();
            lower_host == allowed_clean || lower_host.ends_with(&format!(".{}", allowed_clean))
        });

        if !is_allowed {
            return Err(LumisError::Validation(format!(
                "Host '{}' is not present in the allowed MCP endpoints policy",
                host
            )));
        }
    }

    Ok(())
}

/// Derives a 256-bit AES key from arbitrary master key bytes using SHA-256.
fn derive_aes256_key(master_key: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(master_key);
    hasher.finalize().into()
}

/// Encrypts sensitive MCP credentials using authenticated AES-256-GCM (AEAD).
/// Produces a formatted string: "v1:<base64_nonce>:<base64_ciphertext_and_tag>".
pub fn encrypt_mcp_secret(plaintext: &str, master_key: &[u8]) -> Result<String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let key_bytes = derive_aes256_key(master_key);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| LumisError::Internal(format!("Failed to initialize AES-256-GCM: {}", e)))?;

    // Generate 96-bit (12-byte) cryptographically secure random nonce from UUID v4
    let uuid_bytes = *Uuid::new_v4().as_bytes();
    let nonce_bytes = &uuid_bytes[0..12];
    let nonce = Nonce::from_slice(nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| LumisError::Internal(format!("AES-GCM encryption failed: {}", e)))?;

    let nonce_b64 = BASE64.encode(nonce_bytes);
    let cipher_b64 = BASE64.encode(&ciphertext);

    Ok(format!("v1:{}:{}", nonce_b64, cipher_b64))
}

/// Decrypts sensitive MCP credentials encrypted with AES-256-GCM.
pub fn decrypt_mcp_secret(encrypted: &str, master_key: &[u8]) -> Result<String> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }

    let parts: Vec<&str> = encrypted.split(':').collect();
    if parts.len() != 3 || parts[0] != "v1" {
        return Err(LumisError::Validation("Invalid encrypted secret format".to_string()));
    }

    let nonce_bytes = BASE64
        .decode(parts[1])
        .map_err(|_| LumisError::Validation("Invalid base64 nonce".to_string()))?;
    if nonce_bytes.len() != 12 {
        return Err(LumisError::Validation("Invalid nonce length for AES-GCM".to_string()));
    }

    let cipher_bytes = BASE64
        .decode(parts[2])
        .map_err(|_| LumisError::Validation("Invalid base64 ciphertext".to_string()))?;

    let key_bytes = derive_aes256_key(master_key);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| LumisError::Internal(format!("Failed to initialize AES-256-GCM: {}", e)))?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let decrypted_bytes = cipher
        .decrypt(nonce, cipher_bytes.as_ref())
        .map_err(|_| LumisError::Validation("Authentication tag verification failed or key mismatch".to_string()))?;

    String::from_utf8(decrypted_bytes)
        .map_err(|e| LumisError::Internal(format!("Decrypted secret is not valid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anti_ssrf_rejects_dangerous_endpoints() {
        let empty_allowlist = vec![];
        assert!(validate_mcp_endpoint_url("http://127.0.0.1:8080/mcp", &empty_allowlist).is_err());
        assert!(validate_mcp_endpoint_url("http://localhost:8080/mcp", &empty_allowlist).is_err());
        assert!(validate_mcp_endpoint_url("http://169.254.169.254/latest/meta-data", &empty_allowlist).is_err());
        assert!(validate_mcp_endpoint_url("http://10.0.1.5:9000/sse", &empty_allowlist).is_err());
        assert!(validate_mcp_endpoint_url("http://172.20.0.2:8080/sse", &empty_allowlist).is_err());
        assert!(validate_mcp_endpoint_url("http://192.168.1.100/mcp", &empty_allowlist).is_err());
        assert!(validate_mcp_endpoint_url("ftp://example.com/mcp", &empty_allowlist).is_err());
    }

    #[test]
    fn test_anti_ssrf_allows_legitimate_endpoints() {
        let empty_allowlist = vec![];
        assert!(validate_mcp_endpoint_url("https://api.github.com/mcp", &empty_allowlist).is_ok());
        assert!(validate_mcp_endpoint_url("https://mcp.enterprise.internal/sse", &empty_allowlist).is_ok());

        let strict_allowlist = vec!["github.com".to_string(), "internal.corp".to_string()];
        assert!(validate_mcp_endpoint_url("https://api.github.com/mcp", &strict_allowlist).is_ok());
        assert!(validate_mcp_endpoint_url("https://malicious.evil.com/mcp", &strict_allowlist).is_err());
    }

    #[test]
    fn test_envelope_encryption_roundtrip_and_tamper_proofing() {
        let key = b"super-secret-master-encryption-key";
        let plaintext = "ghp_PersonalAccessTokenSecret123456789";

        let encrypted = encrypt_mcp_secret(plaintext, key).expect("encryption succeeds");
        assert!(encrypted.starts_with("v1:"));

        let decrypted = decrypt_mcp_secret(&encrypted, key).expect("decryption succeeds");
        assert_eq!(decrypted, plaintext);

        // Wrong key must fail authentication
        let wrong_key = b"wrong-key-attempting-to-decrypt";
        assert!(decrypt_mcp_secret(&encrypted, wrong_key).is_err());

        // Tampered ciphertext must fail authentication
        let tampered = format!("{}tamper", encrypted);
        assert!(decrypt_mcp_secret(&tampered, key).is_err());
    }
}
