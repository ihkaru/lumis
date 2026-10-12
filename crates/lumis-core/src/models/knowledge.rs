use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::user::EnterpriseRole;

/// Institutional Document metadata stored in Enterprise Knowledge Vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub tenant_id: String,
    pub title: String,
    pub content_type: String,
    pub min_role: EnterpriseRole,
    pub chunk_count: usize,
    pub created_at: DateTime<Utc>,
}

/// Granular text chunk with optional embedding vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub document_id: String,
    pub tenant_id: String,
    pub chunk_index: usize,
    pub content: String,
    pub min_role: EnterpriseRole,
    pub embedding: Option<Vec<f32>>,
}

/// Request for querying knowledge base with caller security context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    pub query: String,
    pub tenant_id: String,
    pub caller_role: EnterpriseRole,
    pub top_k: usize,
}

/// Ranked search result chunk after Reciprocal Rank Fusion (RRF) & Role-Gated filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeQueryResult {
    pub chunk_id: String,
    pub document_id: String,
    pub content: String,
    pub score: f32,
    pub min_role: EnterpriseRole,
}
