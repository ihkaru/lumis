use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{Skill, SkillSearchResult, SkillSummary};
use sqlx::{Row, SqlitePool};

pub async fn search_skills_fts(
    pool: &SqlitePool,
    tenant_id: &str,
    user_id: &str,
    query: &str,
    limit: usize,
) -> Result<Vec<SkillSearchResult>> {
    let sanitized_query = query.trim().replace('"', "\"\"");
    if sanitized_query.is_empty() {
        return Ok(Vec::new());
    }

    // Prefix matching on the last token to support instant typing autocomplete
    let fts_expr = if sanitized_query.contains(' ') {
        format!("\"{}\"*", sanitized_query)
    } else {
        format!("{}*", sanitized_query)
    };

    let rows = sqlx::query(
        r#"
        SELECT s.id, s.name, s.display_title, s.description, s.category,
               s.always_apply, s.user_invocable, s.version,
               COALESCE(p.is_enabled, 1) as is_enabled,
               bm25(skills_fts) as rank
        FROM skills_fts
        JOIN skills s ON skills_fts.skill_id = s.id
        LEFT JOIN user_skill_preferences p ON p.skill_id = s.id AND p.user_id = ?
        WHERE skills_fts MATCH ? AND (s.tenant_id = ? OR s.tenant_id = 'default' OR s.tenant_id = 'system') AND s.is_active = 1
        ORDER BY rank
        LIMIT ?
        "#,
    )
    .bind(user_id)
    .bind(&fts_expr)
    .bind(tenant_id)
    .bind(limit as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let results = rows
        .into_iter()
        .map(|r| {
            let is_enabled_int: i64 = r.get("is_enabled");
            let always_apply_int: i64 = r.get("always_apply");
            let user_invocable_int: i64 = r.get("user_invocable");
            let rank: f64 = r.get("rank");

            SkillSearchResult {
                skill: SkillSummary {
                    id: r.get("id"),
                    name: r.get("name"),
                    display_title: r.get("display_title"),
                    description: r.get("description"),
                    category: r.get("category"),
                    always_apply: always_apply_int == 1,
                    user_invocable: user_invocable_int == 1,
                    is_enabled: is_enabled_int == 1,
                    version: r.get("version"),
                },
                bm25_score: -rank,
            }
        })
        .collect();

    Ok(results)
}

pub async fn list_skills(
    pool: &SqlitePool,
    tenant_id: &str,
    user_id: &str,
    category: Option<&str>,
    limit: usize,
    offset: usize,
) -> Result<Vec<SkillSummary>> {
    let mut q = String::from(
        r#"
        SELECT s.id, s.name, s.display_title, s.description, s.category,
               s.always_apply, s.user_invocable, s.version,
               COALESCE(p.is_enabled, 1) as is_enabled
        FROM skills s
        LEFT JOIN user_skill_preferences p ON p.skill_id = s.id AND p.user_id = ?
        WHERE (s.tenant_id = ? OR s.tenant_id = 'default' OR s.tenant_id = 'system') AND s.is_active = 1
        "#,
    );

    if category.is_some() {
        q.push_str(" AND s.category = ? ");
    }
    q.push_str(" ORDER BY s.name ASC LIMIT ? OFFSET ?");

    let mut query = sqlx::query(&q).bind(user_id).bind(tenant_id);
    if let Some(cat) = category {
        query = query.bind(cat);
    }
    query = query.bind(limit as i64).bind(offset as i64);

    let rows = query
        .fetch_all(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let is_enabled_int: i64 = r.get("is_enabled");
            let always_apply_int: i64 = r.get("always_apply");
            let user_invocable_int: i64 = r.get("user_invocable");
            SkillSummary {
                id: r.get("id"),
                name: r.get("name"),
                display_title: r.get("display_title"),
                description: r.get("description"),
                category: r.get("category"),
                always_apply: always_apply_int == 1,
                user_invocable: user_invocable_int == 1,
                is_enabled: is_enabled_int == 1,
                version: r.get("version"),
            }
        })
        .collect())
}

pub async fn get_skill_by_name(
    pool: &SqlitePool,
    tenant_id: &str,
    name: &str,
) -> Result<Option<Skill>> {
    let row = sqlx::query(
        r#"
        SELECT id, tenant_id, name, display_title, description, body, frontmatter,
               category, always_apply, user_invocable, disable_model_invocation,
               allowed_tools, author_id, version, is_active, created_at, updated_at
        FROM skills
        WHERE (tenant_id = ? OR tenant_id = 'default' OR tenant_id = 'system') AND name = ? AND is_active = 1
        "#,
    )
    .bind(tenant_id)
    .bind(name)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(row.map(map_row_to_skill))
}

pub async fn get_skill_by_id(
    pool: &SqlitePool,
    tenant_id: &str,
    id: &str,
) -> Result<Option<Skill>> {
    let row = sqlx::query(
        r#"
        SELECT id, tenant_id, name, display_title, description, body, frontmatter,
               category, always_apply, user_invocable, disable_model_invocation,
               allowed_tools, author_id, version, is_active, created_at, updated_at
        FROM skills
        WHERE (tenant_id = ? OR tenant_id = 'default' OR tenant_id = 'system') AND id = ? AND is_active = 1
        "#,
    )
    .bind(tenant_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(row.map(map_row_to_skill))
}

pub async fn get_always_apply_skills(
    pool: &SqlitePool,
    tenant_id: &str,
    user_id: &str,
) -> Result<Vec<Skill>> {
    let rows = sqlx::query(
        r#"
        SELECT s.id, s.tenant_id, s.name, s.display_title, s.description, s.body, s.frontmatter,
               s.category, s.always_apply, s.user_invocable, s.disable_model_invocation,
               s.allowed_tools, s.author_id, s.version, s.is_active, s.created_at, s.updated_at
        FROM skills s
        LEFT JOIN user_skill_preferences p ON p.skill_id = s.id AND p.user_id = ?
        WHERE (s.tenant_id = ? OR s.tenant_id = 'default' OR s.tenant_id = 'system') AND s.always_apply = 1 AND s.is_active = 1 AND COALESCE(p.is_enabled, 1) = 1
        ORDER BY s.name ASC
        "#,
    )
    .bind(user_id)
    .bind(tenant_id)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(rows.into_iter().map(map_row_to_skill).collect())
}

pub(crate) fn map_row_to_skill(r: sqlx::sqlite::SqliteRow) -> Skill {
    let always_apply_int: i64 = r.get("always_apply");
    let user_invocable_int: i64 = r.get("user_invocable");
    let disable_model_int: i64 = r.get("disable_model_invocation");
    let is_active_int: i64 = r.get("is_active");
    let frontmatter_str: String = r.get("frontmatter");
    let allowed_tools_str: String = r.get("allowed_tools");
    let created_at_str: String = r.get("created_at");
    let updated_at_str: String = r.get("updated_at");

    Skill {
        id: r.get("id"),
        tenant_id: r.get("tenant_id"),
        name: r.get("name"),
        display_title: r.get("display_title"),
        description: r.get("description"),
        body: r.get("body"),
        frontmatter: serde_json::from_str(&frontmatter_str).unwrap_or_default(),
        category: r.get("category"),
        always_apply: always_apply_int == 1,
        user_invocable: user_invocable_int == 1,
        disable_model_invocation: disable_model_int == 1,
        allowed_tools: serde_json::from_str(&allowed_tools_str).unwrap_or_default(),
        author_id: r.get("author_id"),
        version: r.get("version"),
        is_active: is_active_int == 1,
        created_at: DateTime::parse_from_rfc3339(&created_at_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_at_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
    }
}
