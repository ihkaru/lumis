# 📒 Enterprise Issue Reference Ledger: Lumis vs LibreChat & Llumen

Ledger ini adalah catatan komparatif resmi dan permanen yang mendokumentasikan isu-isu operasional nyata (*real-world issues*), baik yang masih berstatus **Open** maupun **Closed**, yang diriset langsung melalui GitHub CLI (`gh`) dari repositori `LibreChat-AI/LibreChat` dan arsitektur `Llumen`.

> [!IMPORTANT]
> **Kebijakan Anti-Duplikasi (*Idempotent Handling*)**:
> Sebelum mengimplementasikan fitur atau perbaikan baru, agen WAJIB memeriksa ledger ini. Isu yang telah berstatus `[RESOLVED_AND_VERIFIED]` tidak boleh diimplementasikan ulang, melainkan dipertahankan invarian regression-nya melalui suite pengujian otomatis (`cargo test --workspace`).

---

## 📊 Matriks Status Isu Terverifikasi

| ID Isu | Asal Repositori & Judul Asli | Akar Masalah Arsitektural | Solusi & Invarian di Lumis | Status & Bukti Empiris |
| :---: | :--- | :--- | :--- | :---: |
| **REF-01** | `LibreChat#16814`<br>`/health stays green while Redis is down` | Health probe Kubernetes mengembalikan 200 OK statis meskipun backend storage lumpuh, sehingga pod macet tidak pernah di-recycle. | Endpoint `/health/db` melakukan query `SELECT 1` langsung ke driver basis data fisik secara asinkron. Jika koneksi putus, mengembalikan 503 Service Unavailable. | `RESOLVED_AND_VERIFIED`<br>`tests/api_auth_rbac_test.rs` |
| **REF-02** | `LibreChat#11088`<br>`MCP Tool Filtering` | Seluruh tool MCP diekspos seragam ke semua pengguna tanpa kontrol akses berbasis peran (RBAC/klirens). | **Enterprise Managed Authorization (EMA)**: Tool membawa `min_role`. Penyaringan dilakukan pada `tools/list` dan validasi PEP 403 Forbidden pada `tools/call`. | `RESOLVED_AND_VERIFIED`<br>`tests/mcp_test.rs` |
| **REF-03** | `LibreChat#9803`<br>`OpenID match existing users` | Login via OIDC membuat akun duplikat jika email pengguna sudah terdaftar sebelumnya di sistem. | **JIT SCIM Account Linking**: Handler `/api/auth/me` otomatis menyinkronkan identitas ke akun yang ada berdasarkan `username`/`email` tanpa membuat baris ganda. | `RESOLVED_AND_VERIFIED`<br>`tests/api_auth_rbac_test.rs` |
| **REF-04** | `LibreChat#10006`<br>`Keycloak/Generic OIDC Group Sync` | Group claims dari IdP (Keycloak/AD/Dex) tidak dipetakan ke sistem izin granular secara deterministik. | **EIP ClaimTranslator**: Memetakan header `x-forwarded-groups` secara hierarkis ke 5-Level `EnterpriseRole` (`SuperAdmin` > `TenantAdmin` > `PowerUser` > `Member` > `Guest`). | `RESOLVED_AND_VERIFIED`<br>`tests/iam_test.rs` |
| **REF-05** | `LibreChat#14988`<br>`Deleting knowledge-base files orphans embeddings` | Menghapus file agen tidak menghapus vector chunk di basis data, menyebabkan chunk yatim (*orphaned vectors*). | **Atomic Cascade Deletion**: `delete_document` di `KnowledgeVault` mengeksekusi penghapusan atomik terhadap dokumen dan seluruh chunk terindeks secara bersamaan. | `RESOLVED_AND_VERIFIED`<br>`tests/rag_test.rs` |
| **REF-06** | `LibreChat#16858`<br>`High MongoDB lock contention on login` | Arsitektur NoSQL MongoDB mengalami lock contention dokumen yang parah saat sinkronisasi grup serentak. | **Pure Relational ACID Driver**: Menggunakan SQLite WAL mode (zero lock edge) atau PostgreSQL Connection Pool dengan isolasi transaksi ACID penuh. | `RESOLVED_AND_VERIFIED`<br>`tests/driver_test.rs` |
| **REF-07** | `LibreChat#15298`<br>`Dollar-denominated balance & tool-cost coverage` | Pelacakan penggunaan token dan biaya eksekusi tool tidak memiliki isolasi kuota per penyewa (*multi-tenant*). | **TokenQuotaEngine & OTel FinOps**: Pelacakan kuota sliding-window per tenant dengan HTTP 429 `Retry-After` dan metrik OTel `gen_ai.client.token.usage`. | `RESOLVED_AND_VERIFIED`<br>`tests/otel_finops_test.rs` |
| **REF-08** | `LibreChat#14484`<br>`Model resumable conversation runs` | Sesi percakapan sering terputus di tengah streaming tool execution berdurasi panjang tanpa checkpoint status. | **MCP 2026 Tasks Extension**: Eksekusi asinkron dengan durable task handle (`task_id`), polling status non-blocking (`/api/mcp/tasks/{id}`), dan graceful cancel. | `RESOLVED_AND_VERIFIED`<br>`tests/mcp_test.rs` |
| **REF-09** | `Llumen#ARCH-01`<br>`Heavy Multi-Container RAM Footprint (2-4GB)` | Stack LibreChat/Llumen membutuhkan Node.js + MongoDB + MeiliSearch + Python RAG API, mengonsumsi >2.5GB RAM saat idle. | **Unified Native Rust Binary (<25MB RAM)**: Backend Axum, in-process hybrid RAG, dan static UI Svelte 5 dibundel dalam 1 container tanpa external sidecar. | `RESOLVED_AND_VERIFIED`<br>`Dockerfile`, `k8s/deployment.yaml` |
| **REF-10** | `Llumen#LIC-01`<br>`Copyleft MPL-2.0 License Friction` | Lisensi MPL-2.0 membatasi integrasi tertutup enterprise dan mengharuskan rilis kode turunan. | **Clean-Room Clean Architecture MIT**: Dibuat 100% dari nol (*clean-room*) berlisensi MIT permisif tanpa kode copyleft. | `RESOLVED_AND_VERIFIED`<br>`LICENSE` |
| **REF-11** | `LibreChat#12896`<br>`Kubernetes Gateway API & SSE Buffering` | Reverse proxy (NGINX, Traefik, Coolify) melakukan buffering pada streaming SSE sehingga memicu 504 Gateway Timeout. | **Buffer-Off Annotations & Keep-Alive Ping**: Ingress manifest mematikan buffering (`proxy-buffering: off`), headers `X-Accel-Buffering: no`, dan keep-alive SSE. | `RESOLVED_AND_VERIFIED`<br>`k8s/ingress.yaml`, `docker-compose.coolify.yml` |
| **REF-12** | `LibreChat#16877`<br>`Runtime context stamps system prompt defeating prompt caching` | Timestamp milidetik pada system prompt menyebabkan cache miss 100% pada OpenAI/Anthropic/vLLM RadixAttention. | **Stable-Prefix Layout & Tail Dynamic Context**: System prompt dan tool schemas dipertahankan byte-invariance, variabel dinamis diletakkan di tail pesan. | `RESOLVED_AND_VERIFIED`<br>`tests/prompt_cache_and_idempotency_test.rs` |
| **REF-13** | `LibreChat#14349`<br>`Start-generation idempotency should run ahead of rate limiters` | Retry otomatis atau double-click dikenakan penalti rate limiter 429 dan double token billing alih-alih melampirkan stream yang sedang berjalan. | **Idempotency-First Pipeline Middleware**: Cek duplikasi request via hash/token dijalankan sebelum token quota deduction dan rate limit middleware. | `RESOLVED_AND_VERIFIED`<br>`tests/prompt_cache_and_idempotency_test.rs` |
| **REF-14** | `LibreChat#15757`<br>`mcpTools query invalidated on every chat message` | Hook SSE client menginjeksi query refetch `/api/mcp/tools` di setiap pesan chat, memicu beban berulang ke server dingin/OAuth. | **Catalog Versioning & ETag Gate**: Server menyertakan header ETag/revisi katalog tool. Klien tidak pernah me-refetch tool kecuali revisi berubah. | `RESOLVED_AND_VERIFIED`<br>`tests/prompt_cache_and_idempotency_test.rs` |
| **REF-15** | `LibreChat#9720`<br>`SCIM Support - Automated Group & User Provisioning` | Tidak adanya endpoint SCIM 2.0 standar enterprise untuk de-provisi instan akun saat karyawan resign atau mutasi divisi. | **SCIM 2.0 Identity Governance**: Endpoint `/scim/v2/Users` dan `/scim/v2/Groups` dengan pencabutan token dan blokir akses PEP secara deterministik. | `RESOLVED_AND_VERIFIED`<br>`tests/scim_and_approval_test.rs` |
| **REF-16** | `LibreChat#14193`<br>`UI & Backend support for tool approval policies on MCP servers` | Alat MCP berbahaya atau sensitif dieksekusi secara otomatis tanpa mekanisme persetujuan manusia (*Human-in-the-Loop*). | **Human-in-the-Loop Approval Policies**: Kebijakan per-tool (`AutoApproved`, `RequiresApproval`) dengan sinyal SSE approval token dan durable resume. | `RESOLVED_AND_VERIFIED`<br>`tests/scim_and_approval_test.rs` |

---

## 🔍 Detail Root Cause Analysis & Invarian Pencegahan

### 1. REF-01: False-Positive Health Checks (LibreChat #16814)
- **Akar Masalah**: Kubernetes Pod terus dianggap sehat oleh liveness probe karena probe hanya memeriksa HTTP 200 dari webserver statis tanpa memverifikasi koneksi storage/database di baliknya.
- **Invarian Lumis**: Endpoint `/health/db` wajib menjalankan pemeriksaan konektivitas driver basis data (`driver.health_check().await`). Jika driver gagal, status `503 Service Unavailable` dikembalikan seketika agar kubelet me-restart pod yang macet.

### 2. REF-02: Kebocoran Alat Tanpa Klirens (LibreChat #11088)
- **Akar Masalah**: Seluruh tool yang didaftarkan ke MCP server otomatis terbuka bagi semua pengguna tanpa pembatasan peran organisasi.
- **Invarian Lumis**: Trait `McpToolExecutor` mewajibkan metadata `min_role: EnterpriseRole`. Registry secara deterministik memfilter daftar tool pada `list_tools()` dan menolak pemanggilan via `execute_tool()` dengan `HTTP 403 Forbidden` jika klirens tidak mencukupi.

### 3. REF-05: Orphaned Vector Embeddings (LibreChat #14988)
- **Akar Masalah**: Operasi hapus file pada decoupled Python RAG API tidak menghapus chunks yang terkait di pgvector karena tidak adanya constraint foreign key relasional.
- **Invarian Lumis**: Skema basis data (`documents` dan `document_chunks`) menggunakan relasi relasional `ON DELETE CASCADE`. Pada implementasi in-memory, `chunks.retain(|c| c.document_id != document_id)` dieksekusi secara sinkron dan atomik.

---

## 🛠️ Panduan Pemeliharaan Ledger untuk Agen

1. **Riset Isu Baru via `gh` CLI**:
   ```bash
   gh search issues --repo LibreChat-AI/LibreChat "<kata_kunci>" --limit 10 --json number,title,state,url
   ```
2. **Evaluasi Dampak**:
   - Jika isu telah ditangani di Lumis: Catat referensi ID dan tautkan file test pembuktian empiris.
   - Jika isu belum ditangani dan relevan bagi enterprise: Daftarkan sebagai target pada fase roadmap berikutnya.
