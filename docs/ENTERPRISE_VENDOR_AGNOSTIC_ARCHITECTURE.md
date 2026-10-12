# 🌐 Enterprise Vendor-Agnostic & Zero Lock-in Architecture (October 2026)

> **Executive Overview**: Dokumen ini menetapkan standar arsitektural Lumis untuk menjamin **kebebasan penuh dari keterikatan vendor (*Zero Vendor Lock-in*)** di seluruh lapisan infrastruktur enterprise: *LLM Inference Providers*, *Observability & Telemetry (Sentry/Prometheus/OTel)*, serta *Database Storage*.

---

## 🏛️ Tiga Pilar Arsitektur Tanpa Keterikatan Vendor

```mermaid
flowchart TD
    Client["Klien Enterprise / Web SPA"] --> LumisCore["Lumis Enterprise Core (Axum 0.8)"]
    
    subgraph ObservabilityLayer["Pilar 1: Multi-Vendor Telemetry"]
        LumisCore --> Dispatcher["Composite Telemetry Dispatcher"]
        Dispatcher --> Prom["Prometheus (/metrics)"]
        Dispatcher --> Sentry["Sentry (DSN Breadcrumbs & Errors)"]
        Dispatcher --> OTel["OTel Collector (Datadog/Dynatrace)"]
    end
    
    subgraph InferenceLayer["Pilar 2: Resilient LLM Router & SLA Failover"]
        LumisCore --> Router["Resilient Multi-Provider Router (Circuit Breaker)"]
        Router -->|Prioritas 1 (Sehat)| Primary["Primary Provider (Azure OpenAI / Cloud AI)"]
        Router -->|Failover 429/5xx| Secondary["Secondary Provider (Private vLLM / DeepSeek)"]
        Router -->|Cadangan Darurat| LocalFallback["Local Fallback Cluster"]
    end

    subgraph StorageLayer["Pilar 3: Relational Driver Pattern"]
        LumisCore --> DriverFactory["Database Factory"]
        DriverFactory --> SQLite["SQLite (WAL + mmap / Zero-ops)"]
        DriverFactory --> Postgres["PostgreSQL HA (High Concurrency Cluster)"]
    end
```

---

## 🔌 1. Inference Layer: Driver Pattern & SLA Circuit Breaker

### Kontrak Abstraksi: [`LlmProviderDriver`](file:///root/projects/lumis/crates/lumis-core/src/traits/llm.rs)
Semua komunikasi LLM dilakukan melalui kontrak interface murni tanpa ketergantungan SDK tertutup vendor:
- [`OpenAiCompatibleDriver`](file:///root/projects/lumis/crates/lumis-server/src/llm/openai_compatible.rs): Bekerja universal untuk:
  - **OpenAI & Azure OpenAI**
  - **Private vLLM & Ollama** (Klaster on-premises)
  - **DeepSeek, Groq, Mistral, Together AI**
  - **Cloudflare AI Gateway & LiteLLM Proxy**
- [`ResilientLlmRouter`](file:///root/projects/lumis/crates/lumis-server/src/llm/resilient_router.rs):
  - Memonitor status kesehatan dan tingkat kegagalan (*Circuit Breaker*).
  - Ketika provider utama mengalami lonjakan latensi, *rate limit* HTTP 429, atau *service outage* HTTP 503, router secara otomatis mengalihkan request ke provider cadangan (misal: private vLLM on-premise) dalam sub-100ms tanpa memutus sesi pengguna.

---

## 📊 2. Observability Layer: Multi-Vendor Telemetry (Prometheus, Sentry, OTel)

Lumis menolak keterikatan pada satu format telemetri tertentu:
- Kontrak [`TelemetryExporter`](file:///root/projects/lumis/crates/lumis-core/src/traits/telemetry.rs) memisahkan bisnis dari vendor monitoring:
  1. [`PrometheusExporter`](file:///root/projects/lumis/crates/lumis-server/src/telemetry/prometheus_exporter.rs): Mengekspor metrik time-series berstandar OpenTelemetry GenAI 2026 (`gen_ai.*`) pada endpoint `/metrics`.
  2. [`SentryTelemetryExporter`](file:///root/projects/lumis/crates/lumis-server/src/telemetry/sentry_exporter.rs): Menangkap jejak *LLM failover*, *token breadcrumbs*, serta *error context* ke Sentry DSN tanpa memerlukan library pihak ketiga yang membebani binary.
  3. [`CompositeTelemetryDispatcher`](file:///root/projects/lumis/crates/lumis-server/src/telemetry/composite_dispatcher.rs): Melakukan *fan-out* secara simultan ke seluruh APM yang diaktifkan di enterprise.

---

## ⚙️ Variabel Lingkungan & Konfigurasi Fleksibel

Enterprise dapat mengonfigurasi stack tanpa mengubah satu baris kode pun:

```bash
# === 1. BASIS DATA AGNOSTIK ===
# Gunakan SQLite untuk edge / standalone:
DATABASE_URL=sqlite://lumis.sqlite?mode=rwc
# ATAU gunakan PostgreSQL untuk klaster high-availability:
# DATABASE_URL=postgres://lumis:secret@postgres-cluster.internal:5432/lumis_db

# === 2. RESILIENT LLM ROUTER (MULTI-VENDOR) ===
# Provider Utama (misal Azure OpenAI atau Cloud AI):
LLM_API_BASE=https://my-enterprise-ai.openai.azure.com/openai/deployments/gpt-4o
LLM_API_KEY=my-primary-azure-secret-key

# Provider Cadangan Otomatis (misal Private vLLM / DeepSeek On-Premises):
LLM_BACKUP_API_BASE=http://vllm-cluster.internal:8000/v1
LLM_BACKUP_API_KEY=internal-vllm-token

# === 3. OBSERVABILITAS MULTI-VENDOR ===
# Sentry Error Tracking & LLM Breadcrumbs:
SENTRY_DSN=https://public_key@sentry.enterprise.internal/42

# OpenTelemetry & Prometheus Scraper Port:
PORT=3000 # /metrics tersedia di http://0.0.0.0:3000/metrics
```

---

## 🛡️ Jaminan Mutu & Verifikasi Gauntlet
Seluruh komponen telah teruji dan diverifikasi secara independen:
- Pengujian Failover & Telemetry Fan-out: [`tests/resilient_llm_and_telemetry_test.rs`](file:///root/projects/lumis/crates/lumis-server/tests/resilient_llm_and_telemetry_test.rs)
- Kepatuhan Zero-Mock: Lulus 100% verifikasi `detect-mocks .`
- Kepatuhan Batas Baris: 100% berkas berukuran `< 350 lines` per file.
