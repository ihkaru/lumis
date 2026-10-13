use async_trait::async_trait;
use tokio::sync::mpsc;
use crate::error::Result;
use crate::models::{ChatChunk, ChatCompletionRequest, ModelDescriptor, ProviderHealth};

/// Pluggable LLM Provider Driver Contract.
/// Eliminates vendor lock-in by providing a uniform streaming completion interface
/// across any cloud or self-hosted inference engine (OpenAI, Azure, vLLM, Ollama, Bedrock, Anthropic).
#[async_trait]
pub trait LlmProviderDriver: Send + Sync {
    /// Canonical provider name (e.g. "openai", "azure_openai", "vllm", "ollama", "anthropic").
    fn provider_name(&self) -> &str;

    /// Evaluates provider health and network round-trip latency.
    async fn health_check(&self) -> Result<ProviderHealth>;

    /// Initiates a chat completion streaming tokens asynchronously through an mpsc channel.
    async fn complete_chat(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<mpsc::Receiver<Result<ChatChunk>>>;

    /// Discovers available inference models from this provider.
    async fn list_models(&self) -> Result<Vec<ModelDescriptor>> {
        Ok(Vec::new())
    }
}

