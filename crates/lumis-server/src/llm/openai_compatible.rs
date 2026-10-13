use std::time::Instant;
use async_trait::async_trait;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{
    ChatChunk, ChatCompletionRequest, ModelCapabilities, ModelDescriptor, ProviderHealth,
    ProviderHealthStatus, ToolCallChunk,
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

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature.unwrap_or(0.7),
            "max_tokens": request.max_tokens,
            "stream": true,
        });

        if let Some(ref tools) = request.tools {
            if !tools.is_empty() {
                body["tools"] = serde_json::json!(tools);
                if let Some(ref choice) = request.tool_choice {
                    body["tool_choice"] = choice.clone();
                }
            }
        }

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
                                    continue;
                                }

                                if trimmed.starts_with("data: ") {
                                    let data = trimmed.trim_start_matches("data: ").trim();
                                    if data == "[DONE]" {
                                        let _ = tx.send(Ok(ChatChunk {
                                            content: String::new(),
                                            is_done: true,
                                            model: None,
                                            tool_calls: None,
                                        })).await;
                                        return;
                                    }
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                        let delta = &val["choices"][0]["delta"];
                                        let text_opt = delta["content"].as_str()
                                            .or_else(|| delta["reasoning"].as_str())
                                            .or_else(|| val["choices"][0]["message"]["content"].as_str());

                                        let mut tool_chunks = None;
                                        if let Some(tcs) = delta["tool_calls"].as_array() {
                                            let list: Vec<ToolCallChunk> = tcs.iter().map(|tc| ToolCallChunk {
                                                index: tc["index"].as_u64().unwrap_or(0) as usize,
                                                id: tc["id"].as_str().map(|s| s.to_string()),
                                                name: tc["function"]["name"].as_str().map(|s| s.to_string()),
                                                arguments: tc["function"]["arguments"].as_str().map(|s| s.to_string()),
                                            }).collect();
                                            if !list.is_empty() { tool_chunks = Some(list); }
                                        }

                                        if text_opt.is_some() || tool_chunks.is_some() {
                                            let _ = tx.send(Ok(ChatChunk {
                                                content: text_opt.unwrap_or("").to_string(),
                                                is_done: false,
                                                model: val["model"].as_str().map(|s| s.to_string()),
                                                tool_calls: tool_chunks,
                                            })).await;
                                        }
                                    }
                                } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                    let text_opt = val["choices"][0]["message"]["content"].as_str();
                                    if let Some(text) = text_opt {
                                        let _ = tx.send(Ok(ChatChunk {
                                            content: text.to_string(),
                                            is_done: true,
                                            model: val["model"].as_str().map(|s| s.to_string()),
                                            tool_calls: None,
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

            let _ = tx.send(Ok(ChatChunk {
                content: String::new(),
                is_done: true,
                model: None,
                tool_calls: None,
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
                    if !is_verified_model(id) {
                        continue;
                    }
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
                        let is_reasoning = id.contains("r1") || id.contains("reasoning") || id.contains("5.2");
                        let is_vision = id.contains("vision") || id.contains("5v") || id.contains("m3") || id.contains("k2");
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
                    } else if id.contains("free") {
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

fn is_verified_model(id: &str) -> bool {
    matches!(
        id,
        "cbai/glm-5.2"
            | "cbai/glm-5v-turbo"
            | "cbai/minimax-m3"
            | "cbai/kimi-k2.6"
            | "cbai/kimi-k2.5"
            | "openrouter-free"
            | "my9model-free"
    )
}

fn format_model_display_name(id: &str) -> String {
    match id {
        "my9model-free" => "Nemotron 3 Ultra (Free Gateway)".to_string(),
        "openrouter-free" => "Ling 3.1 Flash (OpenRouter)".to_string(),
        "cbai/glm-5.2" => "GLM 5.2 (Flagship Reasoning)".to_string(),
        "cbai/glm-5v-turbo" => "GLM 5V Turbo (Vision)".to_string(),
        "cbai/minimax-m3" => "MiniMax M3 (1M Long-Context)".to_string(),
        "cbai/kimi-k2.6" => "Kimi K2.6 (Moonshot AI)".to_string(),
        "cbai/kimi-k2.5" => "Kimi K2.5 (High Speed)".to_string(),
        other => other.to_string(),
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

