use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::llm::ChatMessagePayload;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptLayout {
    pub system_prompt: String,
    pub tool_definitions_json: String,
    pub history_messages: Vec<ChatMessagePayload>,
    pub tail_dynamic_context: Option<String>,
}

impl PromptLayout {
    pub fn new(
        system_prompt: impl Into<String>,
        tool_definitions_json: impl Into<String>,
        history_messages: Vec<ChatMessagePayload>,
        tail_dynamic_context: Option<String>,
    ) -> Self {
        Self {
            system_prompt: system_prompt.into(),
            tool_definitions_json: tool_definitions_json.into(),
            history_messages,
            tail_dynamic_context,
        }
    }

    /// Calculates deterministic SHA-256 fingerprint of the stable prefix.
    /// This prefix comprises system instructions and static MCP tool definitions.
    /// Any dynamic variable (e.g. current turn timestamp) is isolated to tail_dynamic_context
    /// ensuring this hash never changes across turns.
    pub fn stable_prefix_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.system_prompt.as_bytes());
        hasher.update(b"||TOOLS||");
        hasher.update(self.tool_definitions_json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Renders an Anthropic-compliant system message with an explicit ephemeral cache breakpoint.
    pub fn to_anthropic_system_block(&self) -> serde_json::Value {
        serde_json::json!([
            {
                "type": "text",
                "text": self.system_prompt,
                "cache_control": { "type": "ephemeral" }
            }
        ])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdempotencyState {
    InFlight,
    Completed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    pub key: String,
    pub user_id: String,
    pub tenant_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub state: IdempotencyState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpCatalogRevision {
    pub version: u64,
    pub etag: String,
}

impl McpCatalogRevision {
    pub fn new(version: u64) -> Self {
        Self {
            version,
            etag: format!("\"mcp-v{}\"", version),
        }
    }
}
