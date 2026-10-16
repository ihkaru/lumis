use chrono::{DateTime, Utc};
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{Skill, SkillSearchResult, SkillSummary};
use sqlx::{PgPool, Row};

pub async fn search_skills_fts(
    pool: &PgPool,
    tenant_id: &str,
    user_id: &str,
    query: &str,
    limit: usize,
) -> Result<Vec<SkillSearchResult>> {
    let sanitized_query = query.trim();
    if sanitized_query.is_empty() {
        return Ok(Vec::new());
    }

    // Prefix matching on tokens for autocomplete support
    let search_term = format!("%{}%", sanitized_query.to_lowercase());

    let rows = sqlx::query(
        r#"
        SELECT s.id, s.name, s.display_title, s.description, s.category,
               s.always_apply, s.user_invocable, s.version,
               COALESCE(p.is_enabled, true) as is_enabled,
               ts_rank(
                   to_tsvector('simple', coalesce(s.name, '') || ' ' || coalesce(s.display_title, '') || ' ' || coalesce(s.description, '') || ' ' || coalesce(s.category, '')),
                   plainto_tsquery('simple', $3)
               ) as rank
        FROM skills s
        LEFT JOIN user_skill_preferences p ON p.skill_id = s.id AND p.user_id = $1
        WHERE (s.tenant_id = $2 OR s.tenant_id = 'default' OR s.tenant_id = 'system')
          AND s.is_active = true
          AND (
              to_tsvector('simple', coalesce(s.name, '') || ' ' || coalesce(s.display_title, '') || ' ' || coalesce(s.description, '') || ' ' || coalesce(s.category, '')) @@ plainto_tsquery('simple', $3)
              OR lower(s.name) LIKE $4
              OR lower(s.display_title) LIKE $4
              OR lower(s.description) LIKE $4
          )
        ORDER BY rank DESC, s.name ASC
        LIMIT $5
        "#,
    )
    .bind(user_id)
    .bind(tenant_id)
    .bind(sanitized_query)
    .bind(&search_term)
    .bind(limit as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    let results = rows
        .into_iter()
        .map(|r| {
            let is_enabled: bool = r.get("is_enabled");
            let always_apply: bool = r.get("always_apply");
            let user_invocable: bool = r.get("user_invocable");
            let rank: f32 = r.try_get("rank").unwrap_or(0.0);

            SkillSearchResult {
                skill: SkillSummary {
                    id: r.get("id"),
                    name: r.get("name"),
                    display_title: r.get("display_title"),
                    description: r.get("description"),
                    category: r.get("category"),
                    always_apply,
                    user_invocable,
                    is_enabled,
                    version: r.get("version"),
                },
                bm25_score: rank as f64,
            }
        })
        .collect();

    Ok(results)
}

pub async fn list_skills(
    pool: &PgPool,
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
               COALESCE(p.is_enabled, true) as is_enabled
        FROM skills s
        LEFT JOIN user_skill_preferences p ON p.skill_id = s.id AND p.user_id = $1
        WHERE (s.tenant_id = $2 OR s.tenant_id = 'default' OR s.tenant_id = 'system') AND s.is_active = true
        "#,
    );

    if category.is_some() {
        q.push_str(" AND s.category = $3 ORDER BY s.name ASC LIMIT $4 OFFSET $5");
        let cat = category.unwrap();
        let rows = sqlx::query(&q)
            .bind(user_id)
            .bind(tenant_id)
            .bind(cat)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await
            .map_err(|e| LumisError::Database(e.to_string()))?;

        Ok(map_summary_rows(rows))
    } else {
        q.push_str(" ORDER BY s.name ASC LIMIT $3 OFFSET $4");
        let rows = sqlx::query(&q)
            .bind(user_id)
            .bind(tenant_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await
            .map_err(|e| LumisError::Database(e.to_string()))?;

        Ok(map_summary_rows(rows))
    }
}

fn map_summary_rows(rows: Vec<sqlx::postgres::PgRow>) -> Vec<SkillSummary> {
    rows.into_iter()
        .map(|r| SkillSummary {
            id: r.get("id"),
            name: r.get("name"),
            display_title: r.get("display_title"),
            description: r.get("description"),
            category: r.get("category"),
            always_apply: r.get("always_apply"),
            user_invocable: r.get("user_invocable"),
            is_enabled: r.get("is_enabled"),
            version: r.get("version"),
        })
        .collect()
}

pub async fn get_skill_by_name(
    pool: &PgPool,
    tenant_id: &str,
    name: &str,
) -> Result<Option<Skill>> {
    let row = sqlx::query(
        r#"
        SELECT id, tenant_id, name, display_title, description, body, frontmatter,
               category, always_apply, user_invocable, disable_model_invocation,
               allowed_tools, author_id, version, is_active, created_at, updated_at
        FROM skills
        WHERE (tenant_id = $1 OR tenant_id = 'default' OR tenant_id = 'system') AND name = $2 AND is_active = true
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
    pool: &PgPool,
    tenant_id: &str,
    id: &str,
) -> Result<Option<Skill>> {
    let row = sqlx::query(
        r#"
        SELECT id, tenant_id, name, display_title, description, body, frontmatter,
               category, always_apply, user_invocable, disable_model_invocation,
               allowed_tools, author_id, version, is_active, created_at, updated_at
        FROM skills
        WHERE (tenant_id = $1 OR tenant_id = 'default' OR tenant_id = 'system') AND id = $2 AND is_active = true
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
    pool: &PgPool,
    tenant_id: &str,
    user_id: &str,
) -> Result<Vec<Skill>> {
    let rows = sqlx::query(
        r#"
        SELECT s.id, s.tenant_id, s.name, s.display_title, s.description, s.body, s.frontmatter,
               s.category, s.always_apply, s.user_invocable, s.disable_model_invocation,
               s.allowed_tools, s.author_id, s.version, s.is_active, s.created_at, s.updated_at
        FROM skills s
        LEFT JOIN user_skill_preferences p ON p.skill_id = s.id AND p.user_id = $1
        WHERE (s.tenant_id = $2 OR s.tenant_id = 'default' OR s.tenant_id = 'system')
          AND s.always_apply = true
          AND s.is_active = true
          AND COALESCE(p.is_enabled, true) = true
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

pub(crate) fn map_row_to_skill(r: sqlx::postgres::PgRow) -> Skill {
    let frontmatter: serde_json::Value = r.try_get("frontmatter").unwrap_or_else(|_| serde_json::json!({}));
    let allowed_tools_val: serde_json::Value = r.try_get("allowed_tools").unwrap_or_else(|_| serde_json::json!([]));
    let allowed_tools: Vec<String> = serde_json::from_value(allowed_tools_val).unwrap_or_default();
    let created_at: DateTime<Utc> = r.get("created_at");
    let updated_at: DateTime<Utc> = r.get("updated_at");

    Skill {
        id: r.get("id"),
        tenant_id: r.get("tenant_id"),
        name: r.get("name"),
        display_title: r.get("display_title"),
        description: r.get("description"),
        body: r.get("body"),
        frontmatter,
        category: r.get("category"),
        always_apply: r.get("always_apply"),
        user_invocable: r.get("user_invocable"),
        disable_model_invocation: r.get("disable_model_invocation"),
        allowed_tools,
        author_id: r.get("author_id"),
        version: r.get("version"),
        is_active: r.get("is_active"),
        created_at,
        updated_at,
    }
}
