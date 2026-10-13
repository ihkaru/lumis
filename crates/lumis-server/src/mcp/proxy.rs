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

/// Loads and registers all database-persisted MCP tools into the runtime registry.
pub async fn sync_servers_to_registry(
    repo: &std::sync::Arc<dyn lumis_core::traits::ChatRepository>,
    registry: &std::sync::Arc<dyn lumis_core::traits::McpRegistry>,
) -> usize {
    let mut count = 0;
    if let Ok(servers) = repo.list_mcp_servers("default").await {
        let http_client = McpHttpClient::new();
        for server in servers {
            if !server.is_enabled {
                continue;
            }
            if let Some(endpoint_url) = server.endpoint_url {
                if let Ok(tools) = repo.list_mcp_tools(&server.id).await {
                    for tool in tools {
                        let schema = serde_json::from_str(&tool.input_schema_json)
                            .unwrap_or_else(|_| serde_json::json!({}));
                        let def = McpToolDefinition {
                            name: tool.name.clone(),
                            description: tool.description.unwrap_or_default(),
                            input_schema: schema,
                            min_role: EnterpriseRole::Member,
                        };
                        let proxy = std::sync::Arc::new(RemoteMcpToolProxy::new(
                            tool.name,
                            def,
                            endpoint_url.clone(),
                            None,
                            http_client.clone(),
                        ));
                        if registry.register_tool(proxy).await.is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}
