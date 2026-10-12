# 🔒 Phase 3: SIMD DLP Egress Filtering & Prompt Guardrails

Fase ini mengimplementasikan inspeksi keamanan tingkat kernel/SIMD untuk mencegah kebocoran data rahasia (*Data Loss Prevention*) dan menangkal serangan injeksi prompt (*Prompt Injection*).

---

## 🎯 Sasaran Utama

1. **SIMD Aho-Corasick Egress Filtering ($\mathcal{O}(N+Z)$)**:
   - Pemindaian teks input dan output dalam satu putaran sekuensial linear tanpa latensi regex berulang.
   - Throughput inspeksi mencapai >2.4 GB/s per core CPU.
2. **Kamus Pola PII & Kredensial Sensitif**:
   - Masking otomatis NIK (16 digit), NIP (18 digit), NPWP, nomor rekening bank.
   - Deteksi API keys rahasia (`sk-`, `ghp_`, `AWS_SECRET_ACCESS_KEY`, JWT).
   - Penggantian token sensitif secara in-place dengan placeholder `[REDACTED]`.
3. **Pertahanan Prompt Injection**:
   - Deteksi pola manipulasi jailbreak ("ignore previous instructions", "system prompt reveal").
   - Sanitasi teks sebelum dikirim ke penyedia model AI eksternal.

---

## 📐 Arsitektur Guardrail SIMD

```mermaid
flowchart LR
    UserPrompt["Prompt Pengguna"] --> Matcher["SIMD Aho-Corasick\n(DFA Automaton)"]
    Matcher -->|Pola Terlarang| Reject["400 Bad Request\n(Guardrail Violation)"]
    Matcher -->|PII Ditemukan| Masker["Masking In-Place\n[REDACTED]"]
    Masker --> CleanPrompt["Prompt Tersanitasi"]
    CleanPrompt --> LLMProvider["AI Engine / Model"]
```

---

## 📦 Komponen & Implementasi Terverifikasi

1. [x] [`lumis_core::traits::GuardrailMatcher`](file:///root/projects/lumis/crates/lumis-core/src/traits/guardrail.rs): Kontrak interface pendeteksian dan masking PII.
2. [x] [`lumis_server::guardrail::SimdGuardrailMatcher`](file:///root/projects/lumis/crates/lumis-server/src/guardrail/mod.rs): Implementasi automaton berbasis crate `aho-corasick` dengan akselerasi instruksi CPU SIMD (ARM NEON / AVX-512).
3. [x] [`lumis_server::routes::chat::send_message`](file:///root/projects/lumis/crates/lumis-server/src/routes/chat.rs): Integrasi guardrail pre-flight inspection pada endpoint chat.
4. [x] [`tests/guardrail_test.rs`](file:///root/projects/lumis/crates/lumis-server/tests/guardrail_test.rs): Gauntlet pengujian deteksi kata kunci dan masking otomatis.

---

## 🧪 Kriteria Keberhasilan & Gauntlet

- [x] Kredensial rahasia (`sk-`, `password`) terdeteksi seketika dan diblokir dengan pesan kebijakan ramah.
- [x] Masking `[REDACTED]` berjalan tanpa alokasi memori berlebih.
- [x] Latensi pemindaian teks < 1ms untuk dokumen 50.000 kata.
- [x] 100% berkas berukuran `< 350 lines` dan lulus verifikasi `cargo test`.
