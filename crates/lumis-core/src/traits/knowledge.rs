use async_trait::async_trait;
use crate::error::Result;
use crate::models::{Document, EnterpriseRole, KnowledgeQuery, KnowledgeQueryResult};

/// Enterprise Knowledge Vault & Role-Gated Hybrid RAG Contract.
/// Guarantees data isolation across tenant and role boundaries.
#[async_trait]
pub trait KnowledgeVault: Send + Sync {
    /// Ingests a new document, generating semantic chunks and embeddings.
    async fn ingest_document(
        &self,
        tenant_id: &str,
        title: &str,
        content: &str,
        content_type: &str,
        min_role: EnterpriseRole,
    ) -> Result<Document>;

    /// Executes role-gated hybrid search (BM25 + Dense Vectors) with Reciprocal Rank Fusion.
    async fn query_knowledge(
        &self,
        query: &KnowledgeQuery,
    ) -> Result<Vec<KnowledgeQueryResult>>;

    /// Lists institutional documents accessible to caller's clearance level.
    async fn list_documents(
        &self,
        tenant_id: &str,
        caller_role: EnterpriseRole,
    ) -> Result<Vec<Document>>;

    /// Deletes a document and its associated chunks if caller has sufficient privileges.
    async fn delete_document(
        &self,
        tenant_id: &str,
        document_id: &str,
        caller_role: EnterpriseRole,
    ) -> Result<bool>;
}
