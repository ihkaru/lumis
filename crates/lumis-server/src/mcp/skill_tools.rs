#![deny(unsafe_code)]

use std::sync::Arc;
use async_trait::async_trait;
use lumis_core::error::Result;
use lumis_core::models::{EnterpriseRole, McpToolCallResult, McpToolDefinition};
use lumis_core::traits::{ChatRepository, McpRegistry, McpToolExecutor};
use serde_json::json;

/// Tool: search_skills
/// Discovers available enterprise skills, guidelines, and directives by keyword or topic.
pub struct SearchSkillsTool {
    repo: Arc<dyn ChatRepository>,
}

impl SearchSkillsTool {
    pub fn new(repo: Arc<dyn ChatRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl McpToolExecutor for SearchSkillsTool {
    fn name(&self) -> &str {
        "search_skills"
    }

    fn definition(&self) -> McpToolDefinition {
        McpToolDefinition {
            name: "search_skills".to_string(),
            description: "Search available enterprise domain skills, guidelines, instructions, or role directives by keyword or query. Returns skill names, descriptions, categories, and tags.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Keywords or search query to find relevant skills (e.g. 'bps-stat-methodology', 'statistics', 'compliance', 'analyst', 'brief')"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Optional maximum number of skills to return (default 5, max 20)"
                    }
                },
                "required": ["query"]
            }),
            min_role: EnterpriseRole::Member,
        }
    }

    async fn execute(&self, _caller_role: EnterpriseRole, arguments: &serde_json::Value) -> Result<McpToolCallResult> {
        let query = arguments.get("query")
            .or_else(|| arguments.get("keyword"))
            .or_else(|| arguments.get("q"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        let limit = arguments.get("limit")
            .and_then(|v| v.as_u64())
            .map(|n| (n as usize).clamp(1, 20))
            .unwrap_or(5);

        let mut results = if !query.is_empty() {
            self.repo.search_skills_fts("default", "", query, limit).await.unwrap_or_default()
        } else {
            Vec::new()
        };

        if results.is_empty() {
            if let Ok(general) = self.repo.list_skills("default", "", None, limit, 0).await {
                results = general.into_iter().map(|s| lumis_core::models::SkillSearchResult {
                    skill: s,
                    bm25_score: 0.0,
                }).collect();
            }
        }

        let formatted: Vec<serde_json::Value> = results
            .into_iter()
            .map(|r| {
                json!({
                    "name": r.skill.name,
                    "display_title": r.skill.display_title,
                    "category": r.skill.category,
                    "description": r.skill.description,
                    "always_apply": r.skill.always_apply,
                    "user_invocable": r.skill.user_invocable,
                    "version": r.skill.version
                })
            })
            .collect();

        let response = json!({
            "query": query,
            "count": formatted.len(),
            "skills": formatted
        });

        Ok(McpToolCallResult {
            content: serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
            is_error: false,
        })
    }
}

/// Tool: get_skill_directive
/// Retrieves complete prompt directives, rules, and guidelines of a specific skill.
pub struct GetSkillDirectiveTool {
    repo: Arc<dyn ChatRepository>,
}

impl GetSkillDirectiveTool {
    pub fn new(repo: Arc<dyn ChatRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl McpToolExecutor for GetSkillDirectiveTool {
    fn name(&self) -> &str {
        "get_skill_directive"
    }

    fn definition(&self) -> McpToolDefinition {
        McpToolDefinition {
            name: "get_skill_directive".to_string(),
            description: "Retrieve the complete instructions, directives, domain rules, and prompt content of a specific skill by its exact name.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The exact name of the skill (e.g. 'bps-stat-methodology', 'data-analyst', 'spip-compliance', 'executive-brief')"
                    }
                },
                "required": ["name"]
            }),
            min_role: EnterpriseRole::Member,
        }
    }

    async fn execute(&self, _caller_role: EnterpriseRole, arguments: &serde_json::Value) -> Result<McpToolCallResult> {
        let raw_name = arguments.get("name")
            .or_else(|| arguments.get("skill_name"))
            .or_else(|| arguments.get("skill"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        let clean_name = raw_name.trim_start_matches('$').trim();

        if clean_name.is_empty() {
            return Ok(McpToolCallResult {
                content: "Error: 'name' parameter is required. Example: {\"name\": \"bps-stat-methodology\"}".to_string(),
                is_error: true,
            });
        }

        match self.repo.get_skill_by_name("default", clean_name).await {
            Ok(Some(skill)) => {
                let payload = json!({
                    "name": skill.name,
                    "display_title": skill.display_title,
                    "category": skill.category,
                    "description": skill.description,
                    "directives": skill.body,
                    "always_apply": skill.always_apply,
                    "user_invocable": skill.user_invocable,
                    "version": skill.version
                });
                Ok(McpToolCallResult {
                    content: serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string()),
                    is_error: false,
                })
            }
            Ok(None) => {
                Ok(McpToolCallResult {
                    content: format!("Skill '{}' not found. Use search_skills to discover valid skill names.", clean_name),
                    is_error: false,
                })
            }
            Err(e) => {
                Ok(McpToolCallResult {
                    content: format!("Error retrieving skill '{}': {}", clean_name, e),
                    is_error: true,
                })
            }
        }
    }
}

/// Helper function to register native skill discovery tools into MCP registry.
pub async fn register_skill_tools(
    registry: &dyn McpRegistry,
    repo: Arc<dyn ChatRepository>,
) -> Result<()> {
    registry.register_tool(Arc::new(SearchSkillsTool::new(repo.clone()))).await?;
    registry.register_tool(Arc::new(GetSkillDirectiveTool::new(repo))).await?;
    Ok(())
}
