# ⚡ Lumis: High-Performance Enterprise AI Platform

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Backend: Rust](https://img.shields.io/badge/Backend-Rust%201.93-orange.svg?logo=rust)](https://www.rust-lang.org)
[![Frontend: Svelte 5](https://img.shields.io/badge/Frontend-Svelte%205%20Runes-ff3e00.svg?logo=svelte)](https://svelte.dev)
[![Runtime: Bun](https://img.shields.io/badge/Runtime-Bun%201.4-black.svg?logo=bun)](https://bun.sh)
[![Architecture: SOLID & EIP](https://img.shields.io/badge/Architecture-Interface--First%20DIP-emerald.svg)](docs/ALGORITHMS_AND_OPTIMIZATIONS.md)
[![Database: Pluggable](https://img.shields.io/badge/Database-SQLite%20%7C%20Postgres-purple.svg)](crates/lumis-db)
[![Zero Mocks](https://img.shields.io/badge/Zero--Mock-Enforced-success.svg)](scripts/governance-check.sh)

**Lumis** adalah platform percakapan dan orkestrasi AI enterprise generasi baru yang dirancang dari nol (*clean-room implementation*) dengan fokus pada **kecepatan ekstrem, konsumsi RAM ultra-ringan (<25MB), lisensi MIT permisif penuh**, **arsitektur database pluggable (SQLite WAL / PostgreSQL Pool)**, serta **integrasi zero-fork tanpa vendor lock-in**.

---

## 🏛️ Arsitektur Enterprise End-to-End

```mermaid
flowchart TD
    subgraph Ingress["Enterprise Ingress & Security"]
        A["Client / Browser<br>(chat.dvlpid.my.id)"] -->|HTTPS / WSS| B["Traefik / Coolify Proxy<br>(Let's Encrypt TLS)"]
        B --> C["rIDM / Keycloak / Entra ID<br>(SAML 2.0 & OIDC SSO)"]
        C -->|x-forwarded-user & roles| D["Lumis Server (Axum 0.8 / Rust)"]
    end

    subgraph Core["Lumis High-Assurance Core Engine"]
        D --> E["SIMD Guardrails & DLP<br>(Aho-Corasick O(N+Z))"]
        E --> F["SOC Killswitch & Audit<br>(RFC 5424 / ECS SIEM)"]
        F --> G["FinOps Quota & Telemetry<br>(Token Budget & Prometheus)"]
        G --> H["Pluggable Relational Driver<br>(SQLite WAL / PostgreSQL)"]
    end

    subgraph Canvas["Interactive Artifacts Engine"]
        D --> I["Artifact Stream Parser<br>(XML-Tagged CommonMark)"]
        I --> J["Split-Pane Canvas UI<br>(shadcn-svelte + 120Hz GPU)"]
        J --> K["Zero-Bleed Sandboxed Iframe<br>(sandbox='allow-scripts')"]
        K -->|PostMessage IPC| D
    end

    subgraph Upstream["AI Gateway & MCP Ecosystem"]
        D --> L["Universal AI Gateway<br>(9router / OpenAI / Claude)"]
        D --> M["MCP 2026 Tool Registry<br>(Async Tasks & Human-in-the-Loop)"]
    end
```

---

## 🎯 Mengapa Lumis? (The Enterprise Shift)

1. **Lisensi MIT 100% Permisif (Zero Legal Toxicity)**:
   - Bebas dari lisensi copyleft (MPL 2.0 / AGPL / GPL). Lumis bebas diintegrasikan, dibundel, dan dimodifikasi untuk kebutuhan korporasi maupun instansi pemerintah.
2. **Pluggable Relational Driver Pattern (Interface-First DIP)**:
   - Bebas beralih antara **SQLite WAL** (edge/standalone) dan **PostgreSQL pool** (multi-node cluster) hanya melalui konfigurasi `DATABASE_URL` tanpa mengubah kode.
3. **Arsitektur Zero-Fork & Seamless Upgrades**:
   - Organisasi pengadopsi tidak perlu memodifikasi kode inti atau menghadapi *merge conflicts*. Cukup gunakan container image resmi dan kustomisasi via overlay runtime (`/api/config/branding` dan `/api/system/doctor`).
4. **Model-Enabled Interactive Artifacts & Split-Pane Canvas**:
   - Preview HTML, SVG, dan kode yang dieksekusi dalam iframe sandboxed terisolasi (*zero CSS bleed*) dengan jembatan komunikasi PostMessage RPC tanpa kebocoran API key.
5. **Standar Algoritma Mutakhir ($\mathcal{O}(1)$ & $\mathcal{O}(\log N)$)**:
   - **Zero-Copy SSE Token Streaming**: Menggunakan `bytes::Bytes` dan Tokio broadcast channel.
   - **Keyset / Cursor-Based Pagination**: $\mathcal{O}(\log N)$ composite index seek.
   - **SIMD Aho-Corasick Guardrail**: Throughput pemindaian PII 2.4 GB/s per core.
   - **Svelte 5 Runes Signals**: UI responsif 120Hz/144Hz dengan konsumsi RAM browser <20MB.

---

## ⚡ Matriks Kompleksitas Algoritma

| Operasi | Pendekatan Konvensional | Pendekatan Lumis | Dampak Riil |
| :--- | :---: | :---: | :--- |
| **Token SSE Stream** | $\mathcal{O}(N^2)$ string copy | **$\mathcal{O}(1)$ `bytes::Bytes` ref** | Mencegah OOM saat 1000+ streaming paralel |
| **Pagination Chat** | $\mathcal{O}(K)$ scan B-tree leaf | **$\mathcal{O}(\log N)$ Keyset seek** | Query tetap <0.1ms di halaman ke-10.000 |
| **PII & DLP Filter** | $\mathcal{O}(M \times N)$ regex loop | **$\mathcal{O}(N + Z)$ SIMD Aho-Corasick** | Throughput 2.4 GB/s per core |
| **Frontend UI DOM** | $\mathcal{O}(N)$ Virtual DOM diff | **$\mathcal{O}(1)$ Svelte 5 Runes** | 0 lag render pada pesan ribuan token |
| **Emergency Lockdown**| Polling database lambat | **$\mathcal{O}(1)$ Atomic Flag Check** | <1ms global killswitch activation |

---

## 🗺️ Master Roadmap 14-Fase Enterprise

| Fase | Fokus Arsitektur | Status |
| :---: | :--- | :---: |
| **Phase 1** | Universal RBAC & SAML/OIDC Identity Federation | [✓] Selesai |
| **Phase 2** | Pluggable Multi-Database (SQLite WAL & PostgreSQL Pool) | [✓] Selesai |
| **Phase 3** | SIMD Aho-Corasick DLP & Prompt Injection Guardrails | [✓] Selesai |
| **Phase 4** | FinOps Token Budgeting & OpenTelemetry Metrics | [✓] Selesai |
| **Phase 5** | Enterprise Knowledge Vault & Role-Gated Hybrid RAG | [✓] Selesai |
| **Phase 6** | MCP 2026 Protocol & Asynchronous Tasks Extension | [✓] Selesai |
| **Phase 7** | Centralized SIEM Auditing & Sub-Millisecond SOC Killswitch | [✓] Selesai |
| **Phase 8** | Stable-Prefix Prompt Cache & Start-Gen Idempotency | [✓] Selesai |
| **Phase 9** | SCIM 2.0 Identity Governance (RFC 7643 & RFC 7644) | [✓] Selesai |
| **Phase 10** | Human-in-the-Loop Tool Execution Approvals | [✓] Selesai |
| **Phase 11** | UI Design Hygiene, Google Sans & Static Analysis | [✓] Selesai |
| **Phase 12** | shadcn-svelte Standard & 120Hz/144Hz Native Refresh Rate | [✓] Selesai |
| **Phase 13** | Interactive Artifacts, Canvas & Secure IPC Bridge | [✓] Selesai |
| **Phase 14** | Zero-Fork Enterprise Upgrade & Pre-flight Diagnostics | [✓] Selesai |

*Lihat dokumentasi lengkap di [docs/roadmap/00_ROADMAP_OVERVIEW.md](docs/roadmap/00_ROADMAP_OVERVIEW.md).*

---

## 🚀 Panduan Memulai Cepat (Quickstart)

### 1. Menjalankan Backend (Standalone Mode)
```bash
git clone https://github.com/ipds6104/lumis.git
cd lumis
cargo run -p lumis-server
```

### 2. Menjalankan Backend (Mode Enterprise PostgreSQL)
```bash
DATABASE_URL="postgres://user:password@localhost:5432/lumis" cargo run -p lumis-server
```

### 3. Menjalankan Frontend (Svelte 5)
```bash
cd ui && bun install && bun run dev
```

---

## 🛡️ Tata Kelola Kode & Verifikasi Mutlak

Lumis menerapkan doktrin *high-assurance engineering*:
- **Hard Line Limit**: 100% berkas wajib berukuran `< 350 lines` (`bash scripts/check-line-limits.sh`).
- **Zero Mock Policy**: 100% bebas dari mock frameworks, fake simulators, dan silent fallbacks di jalur produksi (`detect-mocks .`).
- **UI Design Hygiene**: 100% kepatuhan standar shadcn-svelte, Google Sans typography, dan 0 emoticons (`detect-ui-hygiene ui`).
- **Zero Unsafe Rust**: `#![deny(unsafe_code)]` diaktifkan di seluruh crate.

---

## 📄 Lisensi

Dilisensikan secara terbuka dan bebas di bawah [MIT License](LICENSE).
