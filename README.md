# ⚡ Lumis

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Backend: Rust](https://img.shields.io/badge/Backend-Rust%201.93-orange.svg?logo=rust)](https://www.rust-lang.org)
[![Frontend: Svelte 5](https://img.shields.io/badge/Frontend-Svelte%205%20Runes-ff3e00.svg?logo=svelte)](https://svelte.dev)
[![Runtime: Bun](https://img.shields.io/badge/Runtime-Bun%201.4-black.svg?logo=bun)](https://bun.sh)
[![Architecture: SOLID & EIP](https://img.shields.io/badge/Architecture-Interface--First%20DIP-emerald.svg)](docs/ALGORITHMS_AND_OPTIMIZATIONS.md)
[![Database: Pluggable](https://img.shields.io/badge/Database-SQLite%20%7C%20Postgres%20%7C%20Valkey-purple.svg)](crates/lumis-db)

**Lumis** adalah platform antarmuka AI enterprise generasi baru yang dirancang dari nol (*clean-room implementation*) dengan fokus pada **kecepatan ekstrem, konsumsi RAM ultra-ringan (<25MB), lisensi MIT permisif penuh**, dan **arsitektur database *pluggable* (SQLite WAL, PostgreSQL, Valkey)**.

---

## 🎯 Mengapa Lumis? (The Enterprise Shift)

1. **Lisensi MIT 100% Permisif (Zero Legal Toxicity)**:
   - Tidak menggunakan lisensi copyleft (MPL 2.0 / AGPL / GPL) yang sering ditolak oleh tim legal korporasi dan CISO. Lumis bebas diintegrasikan dan dimodifikasi untuk kebutuhan internal instansi pemerintah maupun enterprise tertutup.
2. **Pluggable Multi-Database Architecture (Interface-First DIP)**:
   - Tidak terkunci (*vendor lock-in*) pada SQLite lokal saja. Lumis mengabstraksikan penyimpanan data melalui trait `ChatRepository`, memungkinkan perpindahan mulus antara **SQLite WAL** (edge/standalone) dan **PostgreSQL cluster** (enterprise high-concurrency) hanya melalui konfigurasi `DATABASE_URL`.
3. **Standar Algoritma Mutakhir 2026 ($\mathcal{O}(1)$ & $\mathcal{O}(\log N)$)**:
   - **Zero-Copy Token Streaming**: Menggunakan `bytes::Bytes` dan Tokio broadcast channel untuk transmisi token SSE tanpa alokasi heap $\mathcal{O}(N^2)$.
   - **Keyset / Cursor-Based Pagination**: Menghilangkan query `OFFSET` $\mathcal{O}(K)$ yang lambat, beralih ke $\mathcal{O}(\log N)$ composite index seek.
   - **SIMD Aho-Corasick Guardrail**: Pemindaian PII dan rahasia data dalam waktu linear $\mathcal{O}(N + Z)$ terlepas dari jumlah pola.
   - **Svelte 5 Runes Fine-Grained Signals**: Menghapus overhead diffing Virtual DOM $\mathcal{O}(N)$, UI responsif 120 FPS dengan konsumsi RAM browser <20MB.

---

## 🏛️ Arsitektur Workspace

```text
lumis/
├── crates/
│   ├── lumis-core/       # Domain models, errors, traits (ChatRepository, GuardrailMatcher)
│   ├── lumis-db/         # Pluggable backends: SQLite WAL, PostgreSQL, DatabaseFactory
│   └── lumis-server/     # Axum 0.8 HTTP / SSE Server, SIMD Guardrail, SSO JIT Mediator
├── ui/                   # Svelte 5 + TypeScript + Tailwind CSS (Runes & Viewport Windowing)
├── docs/                 # Riset algoritma, optimasi memori & analisis Big-O
├── scripts/              # Verifikasi deterministik batas baris (< 350 lines per file)
├── Makefile              # Build, test, lint, dan automation commands
└── LICENSE               # Lisensi MIT Permisif
```

---

## ⚡ Ringkasan Kompleksitas Algoritma

| Operasi | Pendekatan Konvensional | Pendekatan Lumis | Manfaat Riil |
| :--- | :---: | :---: | :--- |
| **Token SSE Stream** | $\mathcal{O}(N^2)$ string copy | **$\mathcal{O}(1)$ `bytes::Bytes` ref** | Mencegah OOM saat 1000+ streaming paralel |
| **Pagination Chat** | $\mathcal{O}(K)$ scan B-tree leaf | **$\mathcal{O}(\log N)$ Keyset seek** | Query tetap <0.1ms di halaman ke-10.000 |
| **PII & DLP Filter** | $\mathcal{O}(M \times N)$ regex loop | **$\mathcal{O}(N + Z)$ SIMD Aho-Corasick** | Throughput 2.4 GB/s per core |
| **Frontend UI DOM** | $\mathcal{O}(N)$ Virtual DOM diff | **$\mathcal{O}(1)$ Svelte 5 Runes** | 0 lag render pada pesan panjang |

*Baca dokumentasi lengkap di [docs/ALGORITHMS_AND_OPTIMIZATIONS.md](docs/ALGORITHMS_AND_OPTIMIZATIONS.md).*

---

## 🚀 Memulai Cepat (Quickstart)

### 1. Menjalankan Backend (Standalone Mode dengan SQLite)
```bash
# Clone repository
git clone https://github.com/ipds6104/lumis.git
cd lumis

# Jalankan server (otomatis inisialisasi SQLite WAL)
cargo run -p lumis-server
```

### 2. Menjalankan Backend (Mode Enterprise dengan PostgreSQL)
```bash
DATABASE_URL="postgres://user:password@localhost:5432/lumis_db" cargo run -p lumis-server
```

### 3. Menjalankan Frontend (Svelte 5)
```bash
cd ui
bun install
bun run dev
```
Akses UI di `http://localhost:5173`.

---

## 🛡️ Tata Kelola & Verifikasi Old-Coder

Lumis menerapkan protokol disiplin ketat:
- **Batas Baris Deterministik**: 100% berkas wajib berukuran `< 350 lines` (`bash scripts/check-line-limits.sh`).
- **Zero Unsafe Rust**: `#![deny(unsafe_code)]` diaktifkan di seluruh crate.
- **Audit Mandiri**: Jalankan `make audit` untuk memverifikasi linter, baris kode, dan seluruh test suites.

---

## 📄 Lisensi
Dilisensikan secara terbuka dan bebas di bawah [MIT License](LICENSE).
