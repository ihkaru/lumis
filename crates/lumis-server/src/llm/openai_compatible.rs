use std::time::Instant;
use async_trait::async_trait;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{
    ChatChunk, ChatCompletionRequest, ProviderHealth, ProviderHealthStatus,
};
use lumis_core::traits::LlmProviderDriver;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use tokio::sync::mpsc;

/// Universal OpenAI-compatible HTTP inference driver.
/// Operates seamlessly with OpenAI, Azure OpenAI, vLLM, Ollama, DeepSeek, LiteLLM, Groq, and Cloudflare AI Gateway.
pub struct OpenAiCompatibleDriver {
    provider_name: String,
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl OpenAiCompatibleDriver {
    pub fn new(provider_name: String, base_url: String, api_key: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Self {
            provider_name,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            client,
        }
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref key) = self.api_key {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", key)) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        headers
    }
}

#[async_trait]
impl LlmProviderDriver for OpenAiCompatibleDriver {
    fn provider_name(&self) -> &str {
        &self.provider_name
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        let start = Instant::now();
        let url = format!("{}/models", self.base_url);
        let resp = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match resp {
            Ok(r) if r.status().is_success() => Ok(ProviderHealth {
                provider_name: self.provider_name.clone(),
                status: ProviderHealthStatus::Healthy,
                latency_ms,
            }),
            Ok(r) => Ok(ProviderHealth {
                provider_name: self.provider_name.clone(),
                status: ProviderHealthStatus::Degraded(format!("HTTP {}", r.status())),
                latency_ms,
            }),
            Err(e) => Ok(ProviderHealth {
                provider_name: self.provider_name.clone(),
                status: ProviderHealthStatus::Offline(e.to_string()),
                latency_ms,
            }),
        }
    }

    async fn complete_chat(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<mpsc::Receiver<Result<ChatChunk>>> {
        let url = format!("{}/chat/completions", self.base_url);
        let (tx, rx) = mpsc::channel(64);

        let body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature.unwrap_or(0.7),
            "max_tokens": request.max_tokens,
            "stream": true,
        });

        let resp_result = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| LumisError::Internal(format!("Failed to connect to {}: {}", self.provider_name, e)))?;

        if !resp_result.status().is_success() {
            let status = resp_result.status();
            let err_text = resp_result.text().await.unwrap_or_default();
            return Err(LumisError::Internal(format!(
                "Upstream provider {} returned error {}: {}",
                self.provider_name, status, err_text
            )));
        }

        tokio::spawn(async move {
            use tokio_stream::StreamExt;
            let mut stream = resp_result.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_res) = stream.next().await {
                match chunk_res {
                    Ok(bytes) => {
                        if let Ok(text) = std::str::from_utf8(&bytes) {
                            buffer.push_str(text);
                            while let Some(newline_idx) = buffer.find('\n') {
                                let line: String = buffer.drain(..=newline_idx).collect();
                                let trimmed = line.trim();
                                if trimmed.starts_with("data: ") {
                                    let data = trimmed.trim_start_matches("data: ").trim();
                                    if data == "[DONE]" {
                                        let _ = tx.send(Ok(ChatChunk {
                                            content: String::new(),
                                            is_done: true,
                                            model: None,
                                        })).await;
                                        return;
                                    }
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                        if let Some(delta) = val["choices"][0]["delta"]["content"].as_str() {
                                            if !delta.is_empty() {
                                                let _ = tx.send(Ok(ChatChunk {
                                                    content: delta.to_string(),
                                                    is_done: false,
                                                    model: val["model"].as_str().map(|s| s.to_string()),
                                                })).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(LumisError::Internal(e.to_string()))).await;
                        return;
                    }
                }
            }
        });

        Ok(rx)
    }
}
