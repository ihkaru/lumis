#![deny(unsafe_code)]

use async_trait::async_trait;
use lumis_core::error::Result;
use lumis_core::models::{EnterpriseRole, McpToolCallResult, McpToolDefinition};
use lumis_core::traits::McpToolExecutor;
use serde_json::Value;

use super::client::McpHttpClient;

/// Dynamic runtime adapter that bridges an MCP tool registered in the database
/// to an external remote server via JSON-RPC 2.0 HTTP transport.
pub struct RemoteMcpToolProxy {
    tool_name: String,
    definition: McpToolDefinition,
    endpoint_url: String,
    auth_header: Option<(String, String)>,
    client: McpHttpClient,
}

impl RemoteMcpToolProxy {
    pub fn new(
        tool_name: String,
        definition: McpToolDefinition,
        endpoint_url: String,
        auth_header: Option<(String, String)>,
        client: McpHttpClient,
    ) -> Self {
        Self {
            tool_name,
            definition,
            endpoint_url,
            auth_header,
            client,
        }
    }
}

#[async_trait]
impl McpToolExecutor for RemoteMcpToolProxy {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn definition(&self) -> McpToolDefinition {
        self.definition.clone()
    }

    async fn execute(
        &self,
        _caller_role: EnterpriseRole,
        arguments: &Value,
    ) -> Result<McpToolCallResult> {
        let auth_ref = self
            .auth_header
            .as_ref()
            .map(|(s, t)| (s.as_str(), t.as_str()));

        self.client
            .call_remote_tool(&self.endpoint_url, &self.tool_name, arguments, auth_ref)
            .await
    }
}
