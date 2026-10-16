use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Canonical Enterprise Skill representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub display_title: String,
    pub description: String,
    pub body: String,
    pub frontmatter: serde_json::Value,
    pub category: String,
    pub always_apply: bool,
    pub user_invocable: bool,
    pub disable_model_invocation: bool,
    pub allowed_tools: Vec<String>,
    pub author_id: String,
    pub version: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lightweight skill projection for fast catalog assembly and client autocomplete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub id: String,
    pub name: String,
    pub display_title: String,
    pub description: String,
    pub category: String,
    pub always_apply: bool,
    pub user_invocable: bool,
    pub is_enabled: bool,
    pub version: i64,
}

/// Request payload to create a new skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkillInput {
    pub name: String,
    pub display_title: Option<String>,
    pub description: String,
    pub body: String,
    pub frontmatter: Option<serde_json::Value>,
    pub category: Option<String>,
    pub always_apply: Option<bool>,
    pub user_invocable: Option<bool>,
    pub disable_model_invocation: Option<bool>,
    pub allowed_tools: Option<Vec<String>>,
}

/// Request payload to update an existing skill with optimistic locking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSkillInput {
    pub expected_version: i64,
    pub display_title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    pub frontmatter: Option<serde_json::Value>,
    pub category: Option<String>,
    pub always_apply: Option<bool>,
    pub user_invocable: Option<bool>,
    pub disable_model_invocation: Option<bool>,
    pub allowed_tools: Option<Vec<String>>,
}

/// Search hit with BM25 rank score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSearchResult {
    pub skill: SkillSummary,
    pub bm25_score: f64,
}

/// Validates kebab-case name pattern (a-z0-9 and hyphens, 1-64 chars).
pub fn is_valid_skill_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
