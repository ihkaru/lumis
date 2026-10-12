# 🔀 Phase 6: Multi-Model Routing, Circuit Breaker & Private Fallback

Fase ini menghadirkan perutean cerdas (*smart model routing*), ketahanan kegagalan jaringan via *circuit breaker*, dan *auto-failover* ke klaster model internal (*on-premise vLLM/Ollama*) saat penyedia cloud mengalami gangguan.

---

## 🎯 Sasaran Utama

1. **Perutean Multi-Penyedia Dinamis (Dynamic Model Routing)**:
   - Dukungan konektor universal ke berbagai penyedia: Anthropic Claude, OpenAI, DeepSeek, Google Gemini, OpenRouter.
   - Pemilihan model otomatis berdasarkan kompleksitas prompt dan status ketersediaan penyedia.
2. **Circuit Breaker & Exponential Backoff**:
   - Deteksi dini kegagalan upstream (HTTP 5xx, timeout, rate limit 429).
   - Transisi status sirkuit: `Closed` -> `Open` -> `Half-Open` untuk mencegah kelebihan beban pada penyedia yang sedang bermasalah.
3. **Auto-Failover ke Private On-Premises Engine**:
   - Jika koneksi internet atau API cloud terputus, rute otomatis dialihkan dalam <200ms ke instance lokal internal (`vLLM`, `TGI`, atau `Ollama`).
   - Layanan tetap berjalan lancar (*Zero Service Downtime*) untuk kebutuhan operasional esensial kantor.

---

## 📐 Arsitektur Ketahanan & Failover

```mermaid
flowchart TD
    Request["Permintaan Pengguna"] --> Router["Model Router (Lumis)"]
    Router --> CB{"Circuit Breaker"}
    CB -->|Sehat (Closed)| CloudProvider["Cloud LLM (DeepSeek / OpenAI)"]
    CB -->|Gangguan (Open)| LocalFallback["Private vLLM / Ollama Cluster (On-Premises)"]
    CloudProvider -->|Gagal / Timeout| FailoverTrigger["Pemicu Failover Otomatis"]
    FailoverTrigger --> LocalFallback
    LocalFallback --> SuccessResponse["Respon Sukses ke Pengguna"]
```

---

## 📦 Komponen & Target Luaran

1. `lumis_core::traits::provider`: Abstraksi trait penyedia model AI (`ModelProvider`).
2. `lumis_server::routing::circuit_breaker`: Mesin circuit breaker berbasis status atomik lock-free.
3. `lumis_server::routing::failover`: Orkestrator failover multi-tingkat.
4. `tests/failover_test.rs`: Simulasi uji kegagalan jaringan cloud dan verifikasi transisi ke private engine.

---

## 🧪 Kriteria Keberhasilan & Gauntlet

- [ ] Transisi failover ke model lokal selesai dalam <300ms tanpa error 500 terlihat oleh pengguna.
- [ ] Sirkuit otomatis pulih ke cloud setelah penyedia utama dinyatakan sehat pada fase `Half-Open`.
- [ ] 100% berkas berukuran `< 350 lines` dan lulus verifikasi `cargo test`.
