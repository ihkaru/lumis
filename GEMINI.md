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
   - Gunakan **Cargo** untuk backend Rust (`cargo build`, `cargo test`, `cargo clippy`).
   - Gunakan **Bun** untuk frontend dan script JS/TS (`bun install`, `bun run dev`, `bun run build`). Jangan pernah menggunakan raw `npm`.

---

## ⚡ Algoritma Mutakhir & Optimasi O-Notation (2026 Standards)

1. **Zero-Copy Streaming**:
   - Menggunakan `bytes::Bytes` dan Tokio broadcast channel untuk transmisi token SSE.
   - Kompleksitas waktu & memori: $\mathcal{O}(1)$ slice sharing tanpa kloning heap $\mathcal{O}(N^2)$.
2. **Keyset / Cursor Pagination**:
   - Menggunakan `WHERE (created_at, id) < ($cursor_time, $cursor_id) LIMIT 20`.
   - Menghindari `OFFSET K` yang $\mathcal{O}(K)$ table scan; beralih ke $\mathcal{O}(\log N)$ B-Tree index seek.
3. **Aho-Corasick SIMD Guardrail Matcher**:
   - Deteksi PII dan kata terlarang dalam $\mathcal{O}(N + Z)$ linear time terlepas dari jumlah pola.
4. **Svelte 5 Runes Fine-Grained Reactivity**:
   - Menghilangkan Virtual DOM diffing $\mathcal{O}(N)$ pada frontend; mutasi localized $\mathcal{O}(1)$ untuk token aktif.
   - Virtualized viewport scrolling $\mathcal{O}(\log N)$ binary search offset, rendering hanya $\mathcal{O}(K)$ node aktif.

---

## 🛠️ Verification & Test Gauntlet

Sebelum menyelesaikan tugas atau membuat commit, jalankan:
```bash
# 1. Cek batasan baris (< 350 baris per file)
bash scripts/check-line-limits.sh

# 2. Cek linter & format Rust
cargo clippy --all-targets -- -D warnings
cargo test

# 3. Cek frontend (jika relevan)
cd ui && bun run check && cd ..
```
