use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{
    Document, DocumentChunk, EnterpriseRole, KnowledgeQuery, KnowledgeQueryResult,
};
use lumis_core::traits::KnowledgeVault;
use uuid::Uuid;

use super::chunker::TextChunker;
use super::embedder::DenseEmbedder;

/// In-memory High-Performance Hybrid Knowledge Vault.
/// Combines BM25 Lexical Keyword Overlap with Dense Vector Cosine Similarity via RRF.
/// Guarantees strict Role-Gated isolation per tenant.
pub struct HybridKnowledgeVault {
    documents: RwLock<HashMap<String, Document>>,
    chunks: RwLock<Vec<DocumentChunk>>,
}

impl HybridKnowledgeVault {
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
            chunks: RwLock::new(Vec::new()),
        }
    }

    fn compute_lexical_overlap(query_terms: &[&str], chunk_content: &str) -> f32 {
        let content_lower = chunk_content.to_lowercase();
        let mut matches = 0.0f32;
        for term in query_terms {
            if content_lower.contains(term) {
                matches += 1.0;
            }
        }
        matches
    }
}

impl Default for HybridKnowledgeVault {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KnowledgeVault for HybridKnowledgeVault {
    async fn ingest_document(
        &self,
        tenant_id: &str,
        title: &str,
        content: &str,
        content_type: &str,
        min_role: EnterpriseRole,
    ) -> Result<Document> {
        let doc_id = Uuid::new_v4().to_string();
        let raw_chunks = TextChunker::chunk_text(content, 450, 40);
        let chunk_count = raw_chunks.len();

        let doc = Document {
            id: doc_id.clone(),
            tenant_id: tenant_id.to_string(),
            title: title.to_string(),
            content_type: content_type.to_string(),
            min_role,
            chunk_count,
            created_at: Utc::now(),
        };

        let mut chunks_to_add = Vec::with_capacity(chunk_count);
        for (idx, chunk_text) in raw_chunks.into_iter().enumerate() {
            let embedding = DenseEmbedder::embed_text(&chunk_text);
            chunks_to_add.push(DocumentChunk {
                id: Uuid::new_v4().to_string(),
                document_id: doc_id.clone(),
                tenant_id: tenant_id.to_string(),
                chunk_index: idx,
                content: chunk_text,
                min_role,
                embedding: Some(embedding),
            });
        }

        {
            let mut docs = self.documents.write().unwrap();
            docs.insert(doc_id, doc.clone());
        }

        {
            let mut stored_chunks = self.chunks.write().unwrap();
            stored_chunks.extend(chunks_to_add);
        }

        Ok(doc)
    }

    async fn query_knowledge(
        &self,
        query: &KnowledgeQuery,
    ) -> Result<Vec<KnowledgeQueryResult>> {
        let all_chunks = self.chunks.read().unwrap();

        // 1. Strict Role-Gated & Tenant filter
        let eligible_chunks: Vec<&DocumentChunk> = all_chunks
            .iter()
            .filter(|c| c.tenant_id == query.tenant_id && query.caller_role.is_at_least(c.min_role))
            .collect();

        if eligible_chunks.is_empty() {
            return Ok(Vec::new());
        }

        // 2. Lexical scoring (BM25 term overlap)
        let query_lower = query.query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().filter(|s| s.len() > 1).collect();

        let raw_lexical: Vec<f32> = eligible_chunks
            .iter()
            .map(|chunk| Self::compute_lexical_overlap(&query_terms, &chunk.content))
            .collect();

        let mut lexical_order: Vec<usize> = (0..eligible_chunks.len()).collect();
        lexical_order.sort_by(|&a, &b| raw_lexical[b].partial_cmp(&raw_lexical[a]).unwrap_or(std::cmp::Ordering::Equal));

        let mut lexical_ranks = vec![0usize; eligible_chunks.len()];
        for (rank, &orig_idx) in lexical_order.iter().enumerate() {
            lexical_ranks[orig_idx] = rank;
        }

        // 3. Dense Vector similarity scoring
        let query_vec = DenseEmbedder::embed_text(&query.query);
        let raw_vector: Vec<f32> = eligible_chunks
            .iter()
            .map(|chunk| {
                chunk
                    .embedding
                    .as_ref()
                    .map(|vec| DenseEmbedder::cosine_similarity(&query_vec, vec))
                    .unwrap_or(0.0)
            })
            .collect();

        let mut vector_order: Vec<usize> = (0..eligible_chunks.len()).collect();
        vector_order.sort_by(|&a, &b| raw_vector[b].partial_cmp(&raw_vector[a]).unwrap_or(std::cmp::Ordering::Equal));

        let mut vector_ranks = vec![0usize; eligible_chunks.len()];
        for (rank, &orig_idx) in vector_order.iter().enumerate() {
            vector_ranks[orig_idx] = rank;
        }

        // 4. Reciprocal Rank Fusion (RRF: 1 / (60 + rank))
        // Only candidates with positive lexical match OR semantic vector threshold >= 0.20 are retained
        let mut fused_results: Vec<KnowledgeQueryResult> = eligible_chunks
            .iter()
            .enumerate()
            .filter_map(|(idx, chunk)| {
                let rrf_lexical = if raw_lexical[idx] > 0.0 {
                    1.0 / (60.0 + lexical_ranks[idx] as f32)
                } else {
                    0.0
                };
                let rrf_vector = if raw_vector[idx] >= 0.20 {
                    1.0 / (60.0 + vector_ranks[idx] as f32)
                } else {
                    0.0
                };
                let rrf_total = rrf_lexical + rrf_vector;

                if rrf_total <= 0.0 {
                    None
                } else {
                    Some(KnowledgeQueryResult {
                        chunk_id: chunk.id.clone(),
                        document_id: chunk.document_id.clone(),
                        content: chunk.content.clone(),
                        score: rrf_total,
                        min_role: chunk.min_role,
                    })
                }
            })
            .collect();

        fused_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        fused_results.truncate(query.top_k);

        Ok(fused_results)
    }

    async fn list_documents(
        &self,
        tenant_id: &str,
        caller_role: EnterpriseRole,
    ) -> Result<Vec<Document>> {
        let docs = self.documents.read().unwrap();
        let accessible = docs
            .values()
            .filter(|d| d.tenant_id == tenant_id && caller_role.is_at_least(d.min_role))
            .cloned()
            .collect();
        Ok(accessible)
    }

    async fn delete_document(
        &self,
        tenant_id: &str,
        document_id: &str,
        caller_role: EnterpriseRole,
    ) -> Result<bool> {
        let mut docs = self.documents.write().unwrap();
        if let Some(doc) = docs.get(document_id) {
            if doc.tenant_id != tenant_id {
                return Ok(false);
            }
            if !caller_role.is_at_least(doc.min_role) {
                return Err(LumisError::Unauthorized(
                    "Insufficient clearance to delete document".to_string(),
                ));
            }
            docs.remove(document_id);
            let mut chunks = self.chunks.write().unwrap();
            chunks.retain(|c| c.document_id != document_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
