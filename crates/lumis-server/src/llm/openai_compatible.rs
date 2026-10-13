use std::time::Instant;
use async_trait::async_trait;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{
    ChatChunk, ChatCompletionRequest, ModelCapabilities, ModelDescriptor, ProviderHealth,
    ProviderHealthStatus,
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

                                if trimmed.is_empty() || trimmed.starts_with(':') {
                                    continue; // Ignore SSE keepalives & comments like : OPENROUTER PROCESSING
                                }

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
                                        let text_opt = val["choices"][0]["delta"]["content"].as_str()
                                            .or_else(|| val["choices"][0]["delta"]["reasoning"].as_str())
                                            .or_else(|| val["choices"][0]["message"]["content"].as_str());

                                        if let Some(delta) = text_opt {
                                            if !delta.is_empty() {
                                                let _ = tx.send(Ok(ChatChunk {
                                                    content: delta.to_string(),
                                                    is_done: false,
                                                    model: val["model"].as_str().map(|s| s.to_string()),
                                                })).await;
                                            }
                                        }
                                    }
                                } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                    // Handle non-prefixed JSON completions or full responses
                                    let text_opt = val["choices"][0]["message"]["content"].as_str()
                                        .or_else(|| val["choices"][0]["delta"]["content"].as_str());

                                    if let Some(text) = text_opt {
                                        let _ = tx.send(Ok(ChatChunk {
                                            content: text.to_string(),
                                            is_done: true,
                                            model: val["model"].as_str().map(|s| s.to_string()),
                                        })).await;
                                        return;
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

            // Signal stream completion if not already terminated by [DONE]
            let _ = tx.send(Ok(ChatChunk {
                content: String::new(),
                is_done: true,
                model: None,
            })).await;
        });

        Ok(rx)
    }

    async fn list_models(&self) -> Result<Vec<ModelDescriptor>> {
        let url = format!("{}/models", self.base_url);
        let resp = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await
            .map_err(|e| LumisError::Internal(format!("Failed to connect to model gateway: {}", e)))?;

        if !resp.status().is_success() {
            return Err(LumisError::Internal(format!(
                "Model gateway returned status HTTP {}",
                resp.status()
            )));
        }

        let body = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|e| LumisError::Internal(format!("Failed to parse models JSON: {}", e)))?;

        let default_model_env = std::env::var("DEFAULT_MODEL").unwrap_or_else(|_| "my9model-free".to_string());
        let mut models = Vec::new();

        if let Some(data) = body["data"].as_array() {
            for item in data {
                if let Some(id) = item["id"].as_str() {
                    let owned_by = item["owned_by"].as_str().map(|s| s.to_string());

                    let caps = if let Some(c) = item.get("capabilities").filter(|v| v.is_object()) {
                        Some(ModelCapabilities {
                            vision: c["vision"].as_bool().unwrap_or(false),
                            tools: c["tools"].as_bool().unwrap_or(false),
                            reasoning: c["reasoning"].as_bool().unwrap_or(false),
                            context_length: c["contextWindow"].as_u64().or_else(|| item["context_length"].as_u64()),
                            max_output: c["maxOutput"].as_u64().or_else(|| item["max_completion_tokens"].as_u64()),
                        })
                    } else {
                        let is_reasoning = id.contains("r1") || id.contains("reasoning") || id.contains("o1") || id.contains("o3") || id.contains("smart");
                        let is_vision = id.contains("vision") || id.contains("vl") || id.contains("5v");
                        Some(ModelCapabilities {
                            vision: is_vision,
                            tools: true,
                            reasoning: is_reasoning,
                            context_length: item["context_length"].as_u64(),
                            max_output: item["max_completion_tokens"].as_u64(),
                        })
                    };

                    let is_reasoning = caps.as_ref().map(|c| c.reasoning).unwrap_or(false);
                    let is_vision = caps.as_ref().map(|c| c.vision).unwrap_or(false);

                    let tier = if is_reasoning {
                        "Reasoning".to_string()
                    } else if is_vision {
                        "Advanced".to_string()
                    } else if id.contains("free") || id.contains("mini") || id.contains("flash") || id.contains("fast") {
                        "Basic".to_string()
                    } else {
                        "Standard".to_string()
                    };

                    let name = format_model_display_name(id);
                    let provider = format_model_provider(id, owned_by.as_deref());
                    let is_default = id == default_model_env;

                    models.push(ModelDescriptor {
                        id: id.to_string(),
                        name,
                        provider,
                        owned_by,
                        tier,
                        capabilities: caps,
                        is_default,
                        can_access: true,
                    });
                }
            }
        }

        Ok(models)
    }
}

fn format_model_display_name(id: &str) -> String {
    match id {
        "my9model-free" => "Standard Free (9Router)".to_string(),
        "my9model-smart" => "Smart Reasoning (9Router)".to_string(),
        "my9model-fast" => "Fast Turbo (9Router)".to_string(),
        "my9model-cooldown" => "Cooldown Fallback (9Router)".to_string(),
        "openrouter-free" => "OpenRouter Free Combo".to_string(),
        "kilo-free" => "Kilo Free Gateway".to_string(),
        "opencode-free" => "OpenCode Free Coding".to_string(),
        "cbai/glm-5.2" => "GLM 5.2 (Flagship Reasoning)".to_string(),
        "cbai/glm-5.1" => "GLM 5.1 (CodeBuddy)".to_string(),
        "cbai/glm-5.0" => "GLM 5.0 (Balanced)".to_string(),
        "cbai/glm-5.0-turbo" => "GLM 5.0 Turbo (Fast)".to_string(),
        "cbai/glm-5v-turbo" => "GLM 5V Turbo (Vision)".to_string(),
        "cbai/glm-4.7" => "GLM 4.7 (Legacy Stable)".to_string(),
        "cbai/minimax-m3" => "MiniMax M3 (1M Long-Context)".to_string(),
        "cbai/minimax-m2.7" => "MiniMax M2.7 (Efficient)".to_string(),
        "cbai/kimi-k2.7" => "Kimi K2.7 (Vision & Tools)".to_string(),
        "cbai/kimi-k2.6" => "Kimi K2.6 (Long-Context)".to_string(),
        "cbai/kimi-k2.5" => "Kimi K2.5 (High Speed)".to_string(),
        "cbai/hy3-preview" => "Hunyuan 3 Preview".to_string(),
        "cbai/deepseek-v4-pro" => "DeepSeek V4 Pro (Reasoning)".to_string(),
        "cbai/deepseek-v4-flash" => "DeepSeek V4 Flash (Fast)".to_string(),
        "cbai/deepseek-v3-2-volc" => "DeepSeek V3.2 Volc".to_string(),
        other => {
            if let Some(stripped) = other.strip_prefix("cbai/") {
                format!("{} (CodeBuddy)", stripped.to_uppercase())
            } else {
                other.to_string()
            }
        }
    }
}

fn format_model_provider(id: &str, owned_by: Option<&str>) -> String {
    if id.starts_with("cbai/") || owned_by == Some("cbai") {
        "CodeBuddy AI".to_string()
    } else if owned_by == Some("combo") || id.starts_with("my9model") {
        "9Router Gateway".to_string()
    } else if let Some(owner) = owned_by {
        owner.to_string()
    } else {
        "OpenAI Compatible".to_string()
    }
}

