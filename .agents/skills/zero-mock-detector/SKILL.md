---
name: zero-mock-detector
description: High-precision static analysis and code-graph AST inspector for detecting mock code, silent mocks, fake latency simulators, and stubbed logic in production pathways per October 2026 standards. Activate whenever auditing codebase integrity, checking for mock leakage, verifying zero-mock compliance, or ensuring production code connects to real drivers.
---

# 🛡️ Zero-Mock & Silent Mock Static Detector

Skill ini menyediakan prosedur dan perkakas statis berbasis **AST Code-Graph Intelligence** untuk mendeteksi secara instan kode tiruan (*mock*), *silent mock*, simulator latensi jaringan palsu, dan *stubbed logic* yang dilarang berada di jalur produksi (*production paths*), sejalan dengan standar rekayasa perangkat lunak enterprise **Oktober 2026**.

---

## 🎯 Kapan Menggunakan Skill Ini

1. **Sebelum Menyelesaikan Tugas Koding / Refactoring**:
   - Memastikan bahwa kode baru tidak menggunakan *mock* atau *stub* tersembunyi yang menggantikan logika nyata.
2. **Saat Menjalankan Audit Kepatuhan & Tata Kelola**:
   - Sebagai gerbang penegakan *Zero-Mock Invariant* di pipeline CI/CD atau script tata kelola (`scripts/governance-check.sh`).
3. **Saat Memeriksa Repositori Baru / Adopsi Enterprise**:
   - Menjamin bahwa lapisan repositori/gateway/driver benar-benar terhubung ke I/O fisik (basis data atau jaringan), bukan sekadar mengembalikan data JSON statis.

---

## 🛠️ CLI Tool Global: `detect-mocks` / `zero-mock-detector`

Tool ini telah terpasang secara global di `/usr/local/bin/detect-mocks` dan `/usr/local/bin/zero-mock-detector`.

### 1. Perintah Standar (Interaktif / Terminal)
```bash
# Pindai direktori saat ini
detect-mocks

# Pindai workspace spesifik
detect-mocks /path/to/project
```

### 2. Mode Strict (Gagal pada Semua Tingkat Temuan)
```bash
detect-mocks --strict
```

### 3. Mode JSON (Otomasi Agen & CI/CD)
```bash
detect-mocks --json
```

---

## 📐 Taksonomi Pelanggaran Mock (Standar Oktober 2026)

| Tingkat | Kategori Rule | Contoh Deteksi | Aksi Mitigasi |
|---|---|---|---|
| 🚨 **CRITICAL** | `MOCK_IMPORT_*` | `import { sinon } from 'sinon'`, `from unittest.mock import MagicMock`, `use mockall::*` di dalam `src/` | Hapus import pengujian dari berkas produksi. Gunakan dependency injection dengan driver nyata. |
| ⚠️ **HIGH** | `RULE_MOCK_SYMBOL_NAME` | `struct MockUserRepository`, `class FakePaymentGateway` di dalam `src/` | Pindahkan ke direktori `tests/` atau ganti dengan implementasi driver nyata. |
| ⚠️ **HIGH** | `RULE_FAKE_LATENCY` | `tokio::time::sleep()`, `time.sleep()`, `setTimeout()` di dalam repository/service/driver | Hapus simulator jeda waktu palsu. Gunakan panggilan I/O nyata atau backoff terkonfigurasi. |
| ⚠️ **HIGH** | `RULE_STUBBED_CODE` | `todo!()`, `unimplemented!()`, `throw new Error("TODO")`, `raise NotImplementedError` | Tuntaskan implementasi logika bisnis atau trait kontrak. |
| 🟡 **MEDIUM** | `RULE_MOCK_BYPASS_FLAG` | `if (process.env.NODE_ENV === 'test') { return dummy; }` atau `MOCK_ENABLED` | Pisahkan konfigurasi pengujian via adapter pattern / config injection, hindari bypass hardcoded. |

---

## 🧠 Sinergi dengan Code-Graph AST Intelligence

Tool ini terintegrasi langsung dengan mesin **`code-graph` (Rust AST & Tree-sitter SQLite)**:
1. **Auto-Index Verification**: Jika `.code-graph/index.db` belum ada atau usang, tool secara otomatis memicu `code-graph incremental-index`.
2. **Kueri Simbol AST Instan**: Membaca tabel `nodes` (fungsi, kelas, struct, signature) secara langsung dalam <20ms tanpa perlu mem-parsing ulang ribuan berkas.
3. **Penyisihan Berkas Uji Cerdas**: Direktori pengujian (`tests/`, `test/`, `spec/`, `benches/`, `evals/`) diizinkan menggunakan mock untuk kebutuhan unit test fixture, sementara berkas produksi (`src/`, `lib/`, `app/`, `ui/src/`) diisolasi secara ketat.

---

## 🧪 Panduan Verifikasi Cepat

Untuk memverifikasi kepatuhan workspace secara 1 baris:
```bash
detect-mocks . && echo "Zero-mock verification passed!"
```
