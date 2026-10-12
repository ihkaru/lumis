use serde::{Deserialize, Serialize};

/// Type classification for interactive and static artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactType {
    Html,
    Svg,
    Markdown,
    Code,
    Json,
    Custom(String),
}

impl ArtifactType {
    pub fn from_mime_or_name(val: &str) -> Self {
        match val.to_lowercase().as_str() {
            "text/html" | "html" => Self::Html,
            "image/svg+xml" | "svg" => Self::Svg,
            "text/markdown" | "markdown" | "md" => Self::Markdown,
            "application/json" | "json" => Self::Json,
            "text/x-python" | "python" | "javascript" | "typescript" | "rust" | "text/x-code" => {
                Self::Code
            }
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Html => "text/html",
            Self::Svg => "image/svg+xml",
            Self::Markdown => "text/markdown",
            Self::Code => "text/x-code",
            Self::Json => "application/json",
            Self::Custom(c) => c.as_str(),
        }
    }
}

/// Specification representing a parsed model artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSpecification {
    pub identifier: String,
    pub artifact_type: String,
    pub title: String,
    pub content: String,
    pub version: u32,
}

/// Actions requested by sandboxed interactive artifact via PostMessage IPC bridge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload", rename_all = "snake_case")]
pub enum ArtifactIpcAction {
    ModelPrompt { prompt: String },
    ToolInvoke { tool: String, args: serde_json::Value },
    Ready,
}

/// Inbound IPC request envelope from sandboxed iframe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactIpcRequest {
    pub request_id: String,
    pub action: ArtifactIpcAction,
}

/// Outbound IPC response envelope to sandboxed iframe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactIpcResponse {
    pub request_id: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Deterministic parser for Lumis artifacts.
pub struct ArtifactParser;

impl ArtifactParser {
    /// Extracts all `<lumis_artifact ...>...</lumis_artifact>` blocks from raw text.
    pub fn parse_artifacts(input: &str) -> Vec<ArtifactSpecification> {
        let mut results = Vec::new();
        let tag_open_prefix = "<lumis_artifact";
        let tag_close = "</lumis_artifact>";

        let mut search_from = 0;
        let mut version_counter = 1;

        while let Some(start_idx) = input[search_from..].find(tag_open_prefix) {
            let actual_start = search_from + start_idx;
            let after_open = actual_start + tag_open_prefix.len();

            if let Some(open_tag_end) = input[after_open..].find('>') {
                let header_str = &input[after_open..after_open + open_tag_end];
                let content_start = after_open + open_tag_end + 1;

                if let Some(close_idx) = input[content_start..].find(tag_close) {
                    let actual_close = content_start + close_idx;
                    let raw_content = &input[content_start..actual_close];

                    // Extract attributes: identifier, type, title
                    let identifier = Self::extract_attribute(header_str, "identifier")
                        .unwrap_or_else(|| format!("artifact-{}", version_counter));
                    let artifact_type = Self::extract_attribute(header_str, "type")
                        .unwrap_or_else(|| "text/html".to_string());
                    let title = Self::extract_attribute(header_str, "title")
                        .unwrap_or_else(|| identifier.clone());

                    let trimmed_content = raw_content
                        .strip_prefix("\r\n")
                        .or_else(|| raw_content.strip_prefix('\n'))
                        .unwrap_or(raw_content);
                    let trimmed_content = trimmed_content
                        .strip_suffix("\r\n")
                        .or_else(|| trimmed_content.strip_suffix('\n'))
                        .unwrap_or(trimmed_content);

                    results.push(ArtifactSpecification {
                        identifier,
                        artifact_type,
                        title,
                        content: trimmed_content.to_string(),
                        version: version_counter,
                    });

                    version_counter += 1;
                    search_from = actual_close + tag_close.len();
                } else {
                    // Malformed or incomplete tag, break
                    break;
                }
            } else {
                break;
            }
        }

        results
    }

    /// Helper to extract string attributes like key="value" or key='value'
    fn extract_attribute(header: &str, key: &str) -> Option<String> {
        let search_pattern = format!("{}=", key);
        let mut idx = 0;

        while let Some(found) = header[idx..].find(&search_pattern) {
            let actual_idx = idx + found + search_pattern.len();
            let remainder = header[actual_idx..].trim_start();

            if remainder.starts_with('"') {
                if let Some(end) = remainder[1..].find('"') {
                    return Some(remainder[1..1 + end].to_string());
                }
            } else if remainder.starts_with('\'') {
                if let Some(end) = remainder[1..].find('\'') {
                    return Some(remainder[1..1 + end].to_string());
                }
            } else {
                let val_end = remainder
                    .find(|c: char| c.is_whitespace() || c == '>')
                    .unwrap_or(remainder.len());
                if val_end > 0 {
                    return Some(remainder[..val_end].to_string());
                }
            }
            idx = actual_idx;
        }

        None
    }
}
