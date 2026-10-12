# ⚡ Lumis Enterprise Governance & Agent Instructions

> [!IMPORTANT]
> **Aturan Wajib Agen AI & Developer**: File ini mendefinisikan standar arsitektural, protokol Old-Coder, batas modularitas, dan tata kelola teknis untuk seluruh pengembangan di repository **Lumis**.

---

## 🏛️ Arsitektur & Prinsip Desain Fundamental

1. **Lisensi MIT Permisif Penuh**:
   - Repository ini 100% independen secara legal dari upstream copyleft (MPL/AGPL/GPL). Seluruh kode baru berlisensi MIT yang aman untuk adopsi skala perusahaan (CISO, Legal, Vendor-neutral).
2. **Interface-First Principle (DIP & SOLID)**:
   - Trait/Interface wajib didefinisikan sebelum menulis implementasi konkret.
   - Core domain logic dilarang keras bergantung langsung pada database atau driver tertentu.
   - Semua layer bisnis berkomunikasi melalui `Arc<dyn Repository>` atau bounded generic.
3. **Pluggable Multi-Database Architecture**:
   - Mendukung multi-driver tanpa mengubah business logic:
     - `SQLite` (Standalone / Zero-ops, WAL + memory mmap)
     - `PostgreSQL` (Enterprise high-concurrency clustering via SQLx)
     - `Valkey / Redis` (Ephemeral session cache, token bucket rate limiter)
4. **Batas Maksimal 350 Baris per File (Hard Constraint)**:
   - **TIDAK BOLEH ADA SATUPUN FILE YANG MELEBIHI 350 BARIS** (`wc -l < 350`).
   - Setiap kali file mendekati 300 baris, wajib dipecah ke dalam submodule terpisah.
   - Wajib diverifikasi dengan `bash scripts/check-line-limits.sh`.
5. **Zero Unsafe Rust & Memory Safety**:
   - `#![deny(unsafe_code)]` wajib diaktifkan pada semua crates Rust (`lib.rs` / `main.rs`).
   - Error handling wajib menggunakan `thiserror` (lib) atau `eyre`/`anyhow` (bin). Dilarang keras `.unwrap()` pada jalur produksi.
6. **Package Manager & Runtime**:
   - Gunakan **Cargo** untuk backend Rust (`cargo build`, `cargo test`).
   - Gunakan **Bun** untuk frontend dan script JS/TS (`bun install`, `bun run dev`, `bun run build`). Jangan pernah menggunakan raw `npm`.

---

## 🧭 Embedded Skills & Protokol Rekayasa

Repository ini dilengkapi skill bawaan yang terpasang di `.agents/skills/`:
- **Old-Coder Protocol** (`.agents/skills/old-coder/SKILL.md`):
  - *Evidence-first development*: Mengelilingi kode dengan executable spec, tipe kuat, dan gauntlet pengujian ketat sebelum refactor.
  - Verifier independen dan batasan baris deterministik.
- **Old-Coder API Protocol** (`.agents/skills/old-coder-api/SKILL.md`):
  - Desain endpoint HTTP/REST/SSE, pagination kursor, idempotency, dan perlindungan breaking changes.

---

## 🗺️ Pelacakan Roadmap Multi-Fase Enterprise

Setiap pengembangan fitur wajib merujuk dan memperbarui dokumen roadmap di `docs/roadmap/`:
- [Phase 1: Universal RBAC & Identity Federation](docs/roadmap/01_PHASE_1_IDENTITY_FEDERATION_AND_RBAC.md)
- [Phase 2: Pluggable Multi-Database & High Availability](docs/roadmap/02_PHASE_2_HIGH_AVAILABILITY_AND_REPLICATION.md)
- [Phase 3: SIMD DLP & Prompt Guardrails](docs/roadmap/03_PHASE_3_ADVANCED_DLP_AND_PROMPT_GUARDRAILS.md)
- [Phase 4: FinOps, Token Budgeting & Telemetry](docs/roadmap/04_PHASE_4_FINOPS_TOKEN_BUDGETING_AND_METRICS.md)
- [Phase 5: Enterprise Knowledge Vault & Hybrid RAG](docs/roadmap/05_PHASE_5_ENTERPRISE_RAG_AND_KNOWLEDGE_VAULT.md)
- [Phase 6: Multi-Model Routing & Private Fallback](docs/roadmap/06_PHASE_6_MODEL_FAILOVER_AND_PRIVATE_VLLM.md)
- [Phase 7: Centralized SIEM Auditing & Compliance](docs/roadmap/07_PHASE_7_SIEM_AUDIT_LOGGING_AND_COMPLIANCE.md)

---

## ⚡ Algoritma Mutakhir & Optimasi O-Notation (2026 Standards)

1. **Zero-Copy Streaming**: Menggunakan `bytes::Bytes` dan Tokio broadcast channel untuk transmisi token SSE ($\mathcal{O}(1)$ atomic ref-count).
2. **Keyset / Cursor Pagination**: `WHERE (created_at, id) < ($cursor_time, $cursor_id) LIMIT 20` ($\mathcal{O}(\log N)$ index seek).
3. **Aho-Corasick SIMD Guardrail Matcher**: Deteksi PII dan kata terlarang dalam $\mathcal{O}(N + Z)$ linear time terlepas dari jumlah pola.
4. **Svelte 5 Runes Fine-Grained Reactivity**: Menghilangkan Virtual DOM diffing $\mathcal{O}(N)$, mutasi localized $\mathcal{O}(1)$ untuk token aktif.

---

## 🛠️ Verification & Test Gauntlet

Sebelum menyelesaikan tugas atau membuat commit, jalankan:
```bash
# 1. Cek batasan baris (< 350 baris per file)
bash scripts/check-line-limits.sh

# 2. Cek kompilasi & test suites Rust
cargo check --workspace
cargo test --workspace

# 3. Cek frontend (Svelte 5 via Bun)
cd ui && bun run check && cd ..
```
