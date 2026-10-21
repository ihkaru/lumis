use std::time::Instant;
use async_trait::async_trait;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{
    ChatChunk, ChatCompletionRequest, ChatMessagePayload, ModelCapabilities, ModelDescriptor, ProviderHealth,
    ProviderHealthStatus, ToolCallChunk,
};
use lumis_core::traits::LlmProviderDriver;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use tokio::sync::mpsc;
use super::model_metadata::{determine_model_tier, format_model_display_name, format_model_provider, is_verified_model};

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
        let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(180))
            .tcp_keepalive(Some(std::time::Duration::from_secs(30))).pool_idle_timeout(Some(std::time::Duration::from_secs(90))).build().unwrap_or_default();

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
        headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("text/event-stream, application/json"));
        if let Some(ref key) = self.api_key {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", key)) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        headers
    }

    fn adapt_messages_for_upstream(messages: &[ChatMessagePayload]) -> Vec<serde_json::Value> {
        let mut adapted = Vec::new();
        let mut system_parts = Vec::new();

        for msg in messages {
            if msg.role == "system" {
                let trimmed = msg.content.trim();
                if !trimmed.is_empty() {
                    system_parts.push(trimmed);
                }
            }
        }

        if !system_parts.is_empty() {
            let combined = system_parts.join("\n\n");
            adapted.push(serde_json::json!({
                "role": "user",
                "content": format!("[SYSTEM DIRECTIVE]\n{}\n[END SYSTEM DIRECTIVE]", combined)
            }));
            adapted.push(serde_json::json!({
                "role": "assistant",
                "content": "Understood. I will strictly follow all system directives, rules, and enterprise constraints."
            }));
        }

        for msg in messages {
            if msg.role != "system" {
                let mut v = serde_json::json!({
                    "role": msg.role,
                    "content": msg.content,
                });
                if let Some(ref tcs) = msg.tool_calls {
                    v["tool_calls"] = serde_json::json!(tcs);
                }
                if let Some(ref cid) = msg.tool_call_id {
                    v["tool_call_id"] = serde_json::json!(cid);
                }
                adapted.push(v);
            }
        }

        adapted
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
        let adapted_messages = Self::adapt_messages_for_upstream(&request.messages);

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": adapted_messages,
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
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);
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
                                        if let Some(err) = val.get("error") {
                                            let msg = err.get("message").and_then(|m| m.as_str()).unwrap_or("Upstream provider error");
                                            let _ = tx.send(Err(LumisError::Internal(format!("Upstream error: {}", msg)))).await;
                                            return;
                                        }
                                        let delta = &val["choices"][0]["delta"];
                                        let text_opt = delta["content"].as_str()
                                            .or_else(|| delta["reasoning_content"].as_str())
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
                    let tier = determine_model_tier(id, is_reasoning, is_vision);

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
