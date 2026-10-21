use chrono::Utc;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::{CreateSkillInput, Skill, UpdateSkillInput};
use sqlx::PgPool;
use uuid::Uuid;

use super::skills::get_skill_by_id;

pub async fn create_skill(
    pool: &PgPool,
    tenant_id: &str,
    author_id: &str,
    input: &CreateSkillInput,
) -> Result<Skill> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let display_title = input.display_title.clone().unwrap_or_else(|| input.name.clone());
    let category = input.category.clone().unwrap_or_else(|| "general".to_string());
    let frontmatter = input.frontmatter.clone().unwrap_or_else(|| serde_json::json!({}));
    let allowed_tools = input.allowed_tools.clone().unwrap_or_default();
    let allowed_tools_val = serde_json::to_value(&allowed_tools).unwrap_or_else(|_| serde_json::json!([]));

    sqlx::query(
        r#"
        INSERT INTO skills (
            id, tenant_id, name, display_title, description, body, frontmatter,
            category, always_apply, user_invocable, disable_model_invocation,
            allowed_tools, author_id, version, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 1, true, $14, $15)
        "#,
    )
    .bind(&id)
    .bind(tenant_id)
    .bind(&input.name)
    .bind(&display_title)
    .bind(&input.description)
    .bind(&input.body)
    .bind(&frontmatter)
    .bind(&category)
    .bind(input.always_apply.unwrap_or(false))
    .bind(input.user_invocable.unwrap_or(true))
    .bind(input.disable_model_invocation.unwrap_or(false))
    .bind(&allowed_tools_val)
    .bind(author_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(Skill {
        id,
        tenant_id: tenant_id.to_string(),
        name: input.name.clone(),
        display_title,
        description: input.description.clone(),
        body: input.body.clone(),
        frontmatter,
        category,
        always_apply: input.always_apply.unwrap_or(false),
        user_invocable: input.user_invocable.unwrap_or(true),
        disable_model_invocation: input.disable_model_invocation.unwrap_or(false),
        allowed_tools,
        author_id: author_id.to_string(),
        version: 1,
        is_active: true,
        created_at: now,
        updated_at: now,
    })
}

pub async fn update_skill(
    pool: &PgPool,
    tenant_id: &str,
    id: &str,
    input: &UpdateSkillInput,
) -> Result<Skill> {
    let now = Utc::now();
    let current = get_skill_by_id(pool, tenant_id, id)
        .await?
        .ok_or_else(|| LumisError::NotFound(format!("Skill '{}' not found", id)))?;

    if current.version != input.expected_version {
        return Err(LumisError::Conflict(format!(
            "Skill version conflict: current is {}, expected {}",
            current.version, input.expected_version
        )));
    }

    let new_title = input.display_title.as_deref().unwrap_or(&current.display_title);
    let new_desc = input.description.as_deref().unwrap_or(&current.description);
    let new_body = input.body.as_deref().unwrap_or(&current.body);
    let new_cat = input.category.as_deref().unwrap_or(&current.category);
    let new_always = input.always_apply.unwrap_or(current.always_apply);
    let new_user_invocable = input.user_invocable.unwrap_or(current.user_invocable);
    let new_disable_model = input.disable_model_invocation.unwrap_or(current.disable_model_invocation);
    let new_allowed_tools = input.allowed_tools.as_ref().unwrap_or(&current.allowed_tools);
    let allowed_tools_val = serde_json::to_value(new_allowed_tools).unwrap_or_else(|_| serde_json::json!([]));
    let new_frontmatter = input.frontmatter.as_ref().unwrap_or(&current.frontmatter);
    let new_version = current.version + 1;

    sqlx::query(
        r#"
        UPDATE skills SET
            display_title = $1,
            description = $2,
            body = $3,
            frontmatter = $4,
            category = $5,
            always_apply = $6,
            user_invocable = $7,
            disable_model_invocation = $8,
            allowed_tools = $9,
            version = $10,
            updated_at = $11
        WHERE id = $12 AND tenant_id = $13
        "#,
    )
    .bind(new_title)
    .bind(new_desc)
    .bind(new_body)
    .bind(new_frontmatter)
    .bind(new_cat)
    .bind(new_always)
    .bind(new_user_invocable)
    .bind(new_disable_model)
    .bind(&allowed_tools_val)
    .bind(new_version)
    .bind(now)
    .bind(id)
    .bind(tenant_id)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    get_skill_by_id(pool, tenant_id, id)
        .await?
        .ok_or_else(|| LumisError::NotFound("Updated skill not found".to_string()))
}

pub async fn delete_skill(pool: &PgPool, tenant_id: &str, id: &str) -> Result<bool> {
    let res = sqlx::query("UPDATE skills SET is_active = false, updated_at = $1 WHERE id = $2 AND tenant_id = $3")
        .bind(Utc::now())
        .bind(id)
        .bind(tenant_id)
        .execute(pool)
        .await
        .map_err(|e| LumisError::Database(e.to_string()))?;
    Ok(res.rows_affected() > 0)
}

pub async fn toggle_user_skill(
    pool: &PgPool,
    user_id: &str,
    skill_id: &str,
    is_enabled: bool,
) -> Result<()> {
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO user_skill_preferences (user_id, skill_id, is_enabled, updated_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(user_id, skill_id) DO UPDATE SET
            is_enabled = EXCLUDED.is_enabled,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(user_id)
    .bind(skill_id)
    .bind(is_enabled)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| LumisError::Database(e.to_string()))?;

    Ok(())
}

pub async fn seed_default_enterprise_skills(
    pool: &PgPool,
    tenant_id: &str,
    author_id: &str,
) -> Result<()> {
    let seeds = [
        (
            "data-analyst",
            "Enterprise Data Analyst",
            "Analisis data statistik dan aggregasi tabular dengan format markdown rapi",
            "# Data Analyst Directive\nKamu adalah ahli data analyst enterprise. Selalu sertakan tabel ringkas dan kalkulasi terverifikasi.",
            "analysis",
            false,
        ),
        (
            "bps-stat-methodology",
            "Metodologi Statistik BPS & Penjaminan Kualitas",
            "Pedoman resmi metodologi survei, sampling frame, Relative Standard Error (RSE), dan prinsip Satu Data Indonesia",
            "# BPS Statistical Methodology Directive\nPedomani standar statistik resmi Republik Indonesia (BPS):\n1. Kerangka sampel (sampling frame) probabilitas acak berlapis (stratified random sampling).\n2. Penghitungan Relative Standard Error (RSE) dengan ambang batas keandalan (RSE < 25%).\n3. Pengendalian non-sampling error dan imputation methodology.\n4. Kepatuhan terhadap Prinsip Satu Data Indonesia (SDI).",
            "bps",
            true,
        ),
        (
            "spip-compliance",
            "Kepatuhan SPIP & Manajemen Risiko",
            "Pedoman Sistem Pengendalian Intern Pemerintah (SPIP) terintegrasi dan audit kepatuhan",
            "# SPIP Compliance Directive\nPedomani prinsip audit SPIP:\n1. Pastikan kepatuhan terhadap prosedur baku pengendalian intern.\n2. Verifikasi risiko operasional dan mitigasi dampak.\n3. Catat jejak audit secara transparan.",
            "compliance",
            true,
        ),
        (
            "executive-brief",
            "Executive Briefing & Policy Summary",
            "Penyusunan telaahan staf, nota dinas, dan ringkasan eksekutif berkepadatan informasi tinggi",
            "# Executive Brief Directive\nSusun laporan berkepadatan informasi tinggi untuk pimpinan eksekutif:\n1. Ringkasan Eksekutif (maksimal 3 paragraf padat poin inti).\n2. Fakta Kunci & Data Indikator Utama.\n3. Implikasi Kebijakan & Risiko Strategis.\n4. Opsi Keputusan yang Direkomendasikan dengan pro dan kontra jelas.",
            "management",
            false,
        ),
        (
            "openui-builder",
            "Interactive OpenUI & Generative Widget Architect",
            "Spesifikasi deklaratif OpenUI AST untuk pembuatan kalkulator interaktif, dashboard KPI, formulir, slider, stepper, dan widget reaktif tanpa server execution.",
            "# OpenUI Builder Directive\nLumis Generative UI Architecture mendukung dua format:\n1. OpenUI DSL (```openui): Untuk kalkulator, slider, dan form reaktif client-side cepat.\n   Aturan: Awali `root = Col(...)` atau `root = Card(...)` pada baris pertama. Gunakan literal `$var = val` untuk state reaktif. Hubungkan input `Slider($var, min, max, step, label)`, `Input($var, label, type)`, `SegmentedControl($var, options)`, `Checkbox($var, label)`. Gunakan fungsi @ (@Round, @Clamp, @Format, @Percent, @Sum, @Avg, @Default) dan tombol `@Set($var, val)`, `@Reset($var)`, `@CopyToClipboard(...)`, `@ToAssistant(\"...\" )`.\n2. Interactive Web Apps & Dashboards (```html:preview atau ```html): Untuk dashboard visual, chart, formulir multi-langkah, dan aplikasi interaktif lengkap. Didukung Tailwind CSS, Lucide icons, dan shadcn design tokens di dalam sandbox iframe.\nDILARANG memanggil backend execution tools (python/bash/execute_code) untuk membuat UI. Selalu outputkan blok ```openui atau ```html:preview secara langsung.",
            "frontend",
            false,
        ),
    ];

    for (name, title, desc, body, cat, always) in seeds {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM skills WHERE name = $1 AND (tenant_id = $2 OR tenant_id = 'default' OR tenant_id = 'system')"
        )
        .bind(name)
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if exists > 0 {
            continue;
        }

        let input = CreateSkillInput {
            name: name.to_string(),
            display_title: Some(title.to_string()),
            description: desc.to_string(),
            body: body.to_string(),
            frontmatter: None,
            category: Some(cat.to_string()),
            always_apply: Some(always),
            user_invocable: Some(true),
            disable_model_invocation: Some(false),
            allowed_tools: Some(Vec::new()),
        };
        let _ = create_skill(pool, tenant_id, author_id, &input).await;
    }

    Ok(())
}
