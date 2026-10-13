#![deny(unsafe_code)]

use std::time::Duration;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{EnterpriseRole, McpToolCallResult, McpToolDefinition};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};

/// Production-grade asynchronous HTTP client for remote Model Context Protocol (MCP) servers.
/// Communicates via JSON-RPC 2.0 over HTTP / Server-Sent Events (SSE).
#[derive(Clone)]
pub struct McpHttpClient {
    client: reqwest::Client,
}

impl Default for McpHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl McpHttpClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    fn build_headers(auth_header: Option<(&str, &str)>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some((scheme, token)) = auth_header {
            if !token.is_empty() {
                let formatted = if scheme.eq_ignore_ascii_case("bearer") {
                    format!("Bearer {}", token)
                } else if scheme.eq_ignore_ascii_case("basic") {
                    format!("Basic {}", token)
                } else {
                    token.to_string()
                };

                if let Ok(val) = HeaderValue::from_str(&formatted) {
                    headers.insert(AUTHORIZATION, val);
                }
            }
        }

        headers
    }

    /// Automatically resolves SSE endpoint if URL ends with `/sse` or if SSE handshake announces endpoint.
    pub async fn resolve_post_endpoint(&self, endpoint_url: &str, headers: &HeaderMap) -> String {
        if !endpoint_url.ends_with("/sse") {
            return endpoint_url.to_string();
        }

        let mut sse_headers = headers.clone();
        sse_headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("text/event-stream"));

        if let Ok(mut res) = self.client.get(endpoint_url).headers(sse_headers).send().await {
            if let Ok(Some(chunk)) = res.chunk().await {
                let text = String::from_utf8_lossy(&chunk);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if let Some(rest) = trimmed.strip_prefix("data:") {
                        let path = rest.trim();
                        if path.starts_with("http://") || path.starts_with("https://") {
                            return path.to_string();
                        } else if path.starts_with('/') {
                            if let Ok(base) = reqwest::Url::parse(endpoint_url) {
                                if let Ok(joined) = base.join(path) {
                                    return joined.to_string();
                                }
                            }
                        }
                    }
                }
            }
        }

        endpoint_url.trim_end_matches("/sse").to_string() + "/mcp"
    }

    /// Performs pre-flight handshake (`initialize` and `tools/list`) to discover remote tools.
    pub async fn ping_and_discover(
        &self,
        endpoint_url: &str,
        auth_header: Option<(&str, &str)>,
    ) -> Result<Vec<McpToolDefinition>> {
        let headers = Self::build_headers(auth_header);
        let target_url = self.resolve_post_endpoint(endpoint_url, &headers).await;

        // 1. JSON-RPC initialize handshake
        let init_payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": true }
                },
                "clientInfo": {
                    "name": "lumis-core-gateway",
                    "version": "1.0.0"
                }
            }
        });

        let init_res = self
            .client
            .post(&target_url)
            .headers(headers.clone())
            .json(&init_payload)
            .send()
            .await
            .map_err(|e| LumisError::Internal(format!("MCP initialize request failed: {}", e)))?;

        if !init_res.status().is_success() {
            return Err(LumisError::Internal(format!(
                "MCP server returned HTTP {} during initialize handshake",
                init_res.status()
            )));
        }

        // 2. Query available tools via tools/list
        let list_payload = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let list_res = self
            .client
            .post(&target_url)
            .headers(headers)
            .json(&list_payload)
            .send()
            .await
            .map_err(|e| LumisError::Internal(format!("MCP tools/list request failed: {}", e)))?;

        if !list_res.status().is_success() {
            return Err(LumisError::Internal(format!(
                "MCP server returned HTTP {} during tools/list query",
                list_res.status()
            )));
        }

        let body: Value = list_res
            .json()
            .await
            .map_err(|e| LumisError::Serialization(format!("Invalid JSON in tools/list response: {}", e)))?;

        let tools_array = body
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();

        let mut discovered = Vec::with_capacity(tools_array.len());
        for t in tools_array {
            if let Some(name) = t.get("name").and_then(|n| n.as_str()) {
                let description = t
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();
                let input_schema = t
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| json!({"type": "object"}));

                discovered.push(McpToolDefinition {
                    name: name.to_string(),
                    description,
                    input_schema,
                    min_role: EnterpriseRole::Member, // default accessible to all authenticated members
                });
            }
        }

        Ok(discovered)
    }

    /// Invokes a specific remote tool via JSON-RPC 2.0 `tools/call`.
    pub async fn call_remote_tool(
        &self,
        endpoint_url: &str,
        tool_name: &str,
        arguments: &Value,
        auth_header: Option<(&str, &str)>,
    ) -> Result<McpToolCallResult> {
        let headers = Self::build_headers(auth_header);
        let target_url = self.resolve_post_endpoint(endpoint_url, &headers).await;

        let call_payload = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        let res = self
            .client
            .post(&target_url)
            .headers(headers)
            .json(&call_payload)
            .send()
            .await
            .map_err(|e| LumisError::Internal(format!("MCP tools/call failed: {}", e)))?;

        if !res.status().is_success() {
            return Ok(McpToolCallResult {
                content: format!("MCP HTTP error status: {}", res.status()),
                is_error: true,
            });
        }

        let body: Value = res
            .json()
            .await
            .map_err(|e| LumisError::Serialization(format!("Invalid JSON in tools/call response: {}", e)))?;

        if let Some(err) = body.get("error") {
            let msg = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown JSON-RPC error");
            return Ok(McpToolCallResult {
                content: msg.to_string(),
                is_error: true,
            });
        }

        let result_obj = body.get("result");
        let is_error = result_obj
            .and_then(|r| r.get("isError"))
            .and_then(|b| b.as_bool())
            .unwrap_or(false);

        let content_str = if let Some(contents) = result_obj.and_then(|r| r.get("content")).and_then(|c| c.as_array()) {
            let texts: Vec<String> = contents
                .iter()
                .filter_map(|item| {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        Some(text.to_string())
                    } else {
                        Some(item.to_string())
                    }
                })
                .collect();
            texts.join("\n")
        } else if let Some(r) = result_obj {
            r.to_string()
        } else {
            "{}".to_string()
        };

        Ok(McpToolCallResult {
            content: content_str,
            is_error,
        })
    }
}
