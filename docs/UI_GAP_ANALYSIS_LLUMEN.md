# 📊 Lumis vs Llumen UI Comparative Matrix & Bug Audit Ledger
> **Living Reference Ledger for UI Parity & Defect Resolution**  
> Standar Arsitektur: Svelte 5, shadcn-svelte, 120Hz/144Hz GPU Compositor, Zero Emoticons.

---

## 🏛️ Executive Summary

Berdasarkan audit komparatif mendalam antara **Llumen Frontend** (`/root/projects/llumen/frontend/src/`) dan **Lumis UI** (`/root/projects/lumis/ui/src/`), ditemukan bahwa Lumis UI saat ini masih berupa *single-file prototype* (`App.svelte`, 309 baris) dengan **48 kesenjangan fungsional (gaps)** dan **12 bug kritis/defek fungsional**. 

Sebaliknya, Llumen memiliki arsitektur modular multi-rute (50+ komponen atomik), streaming SSE tingkat lanjut dengan AbortController, perenderan Markdown berkinerja tinggi tanpa lag DOM, dukungan KaTeX matematika, diagram Mermaid, inspektur penalaran (*reasoning*) `<think>`, manajemen sesi lengkap (pencarian, rename, pin, hapus), hingga modal preferensi pengguna multi-tema.

---

## 🧭 Matrix 1: Rute Halaman & Navigasi (Routing & Page Lifecycle)

| Rute / Halaman | Fitur Llumen (`routes/`) | Status Lumis UI (`ui/src/`) | Klasifikasi Defek | Dampak Operasional & Kesenjangan |
| :--- | :--- | :--- | :---: | :--- |
| **`/` (Root)** | Redirect otomatis ke `/chat/new` atau sesi terakhir jika login; splash screen jika unauthenticated. | Hanya memuat sesi pertama dari API atau array kosong di satu file `App.svelte`. | 🚨 **CRITICAL BUG** | Tidak ada URL state. Tombol Back/Forward browser tidak berfungsi. Refresh selalu mereset tampilan. |
| **`/login`** | Formulir login lengkap (`username`, `password`, status pending inert, error toast, retry, i18n). | **TIDAK ADA**. Bergantung 100% pada reverse proxy header `x-forwarded-user`. | 🚨 **CRITICAL DEFECT** | Pengguna langsung atau mode dev mandiri tanpa proxy tidak dapat login (hanya melihat teks *Connecting...* tanpa aksi). |
| **`/chat/new`** | Halaman selamat datang dengan tipografi gradien besar, kartu starter prompts, dan pemilihan mode chat. | Mengosongkan viewport pesan tanpa panduan interaktif atau starter prompts. | ⚠️ **HIGH GAP** | Pengalaman pertama (*onboarding*) dingin dan tidak ramah pengguna baru. |
| **`/chat/[id]`** | Stateful dynamic route dengan ID di URL (`/chat/123`), deep-linking, bookmarking, dan perbaruan `<title>` dinamis. | **TIDAK ADA**. ID sesi hanya disimpan di variabel memori `$state(activeSessionId)`. | 🚨 **CRITICAL BUG** | Pengguna tidak dapat membagikan URL percakapan, menandai (bookmark), atau memuat ulang halaman tanpa kehilangan konteks. |
| **`/markdown`** | Halaman terisolasi untuk preview & rendering scratchpad dokumen Markdown secara mandiri. | **TIDAK ADA**. | 🟡 **MEDIUM GAP** | Ketiadaan tempat pengujian atau pratinjau format dokumen kompleks secara terpisah. |
| **`+error.svelte`** | Error Boundary bawaan untuk menangkap exception runtime Svelte dan tombol pemulihan *Retry*. | **TIDAK ADA**. | 🚨 **CRITICAL DEFECT** | Jika terjadi galat JavaScript pada satu komponen, seluruh aplikasi mengalami *White Screen of Death* (WSOD). |

---

## 💬 Matrix 2: Viewport Pesan & Perenderan Konten (Message Viewport)

| Sub-Komponen | Kemampuan Llumen (`message/`) | Status Lumis UI | Klasifikasi Defek | Dampak Operasional & Kesenjangan |
| :--- | :--- | :--- | :---: | :--- |
| **Markdown Parser** | Parser streaming kustom (`smd.ts` + `renderer.ts`) yang memproses delta token tanpa re-render pohon DOM utuh. | Teks mentah dalam `<div>{cleanText}</div>`. | 🚨 **CRITICAL BUG** | Markdown (`**bold**`, `# judul`, `- daftar`, tabel) tampil sebagai karakter mentah tanpa format! |
| **Code Highlighting** | `Code.svelte` dengan pewarnaan sintaksis, badge bahasa pemrograman, dan tombol salin 1-klik (`clickOnCopy`). | Teks mentah tanpa background, tanpa warna sintaksis, tanpa tombol salin. | 🚨 **CRITICAL BUG** | Kode program tidak terbaca rapi dan menyulitkan developer untuk menyalin snippet. |
| **KaTeX Math Engine** | `Latex.svelte` & `latex.ts` untuk rumus inline (`$..$`) dan rumus blok (`$$..$$`). | Teks mentah kode LaTeX. | ⚠️ **HIGH DEFECT** | Respon ilmiah dan matematis tidak terformat dengan benar. |
| **Mermaid Engine** | `mermaid/` renderer dinamis dengan zoom dan penyesuaian tema. | Teks mentah kode diagram. | ⚠️ **HIGH DEFECT** | Diagram alur dan arsitektur gagal divisualisasikan. |
| **Reasoning `<think>`** | `Reasoning.svelte` dengan accordion `bits-ui`, ikon Brain, render karakter dinamis, dan auto-collapse. | Tag `<think>` mentah DeepSeek/QwQ dicetak langsung ke dalam chat bubble. | 🚨 **CRITICAL BUG** | Obrolan dipenuhi ribuan baris proses berpikir internal model yang mengacaukan antarmuka pengguna. |
| **Tool / MCP Execution** | `ToolBox.svelte` & `Tool.svelte` menampilkan nama tool, parameter input, status running, dan hasil payload. | **TIDAK DITAMPILKAN**. Panggilan fungsi/MCP tidak memiliki elemen visual. | ⚠️ **HIGH DEFECT** | Pengguna tidak dapat melihat aksi otonom model atau memverifikasi query data yang dipanggil. |
| **Deep Research** | `DeepResearch.svelte` & `ResearchStep.svelte` dengan visualisasi langkah penelitian bertahap. | **TIDAK ADA**. | ⚠️ **HIGH GAP** | Alur penelitian multi-step agen tidak terlihat. |
| **Web Citations** | `Citations.svelte` menampilkan favicon, judul sumber, hostname, dan URL eksternal terformat rapi. | **TIDAK ADA**. Sumber RAG tercetak sebagai URL mentah. | ⚠️ **HIGH GAP** | Pengguna tidak dapat mengecek rujukan atau kredibilitas sumber informasi dengan mudah. |
| **Aksi Pesan** | Salin pesan, edit prompt user (`ResponseEdit`), regenerasi asisten, cabang varian (`Pagination.svelte`). | **TIDAK ADA AKSI**. | 🚨 **CRITICAL GAP** | Pengguna tidak dapat mengulang prompt, mengubah prompt, atau menyalin respon asisten. |
| **Metrik Inferensi** | Kecepatan token (tokens/detik), total token, durasi waktu inferensi. | **TIDAK ADA**. | 🟡 **MEDIUM GAP** | Ketiadaan observabilitas performa model pada UI pengguna. |
| **Scroll Lock & Minimap**| `Minimap.svelte` untuk lompatan pesan cepat, dan kunci auto-scroll cerdas saat pengguna menggulir ke atas. | Pengguliran dasar tanpa penguncian; teks baru mendorong layar secara agresif saat dibaca. | ⚠️ **HIGH BUG** | Layar melompat-lompat (*jittery scrolling*) saat pengguna membaca pesan lama selagi teks baru mengalir. |

---

## ⌨️ Matrix 3: Composer Input & Interaktivitas (Input Bar)

| Fitur Input | Kemampuan Llumen (`input/`) | Status Lumis UI | Klasifikasi Defek | Dampak Operasional & Kesenjangan |
| :--- | :--- | :--- | :---: | :--- |
| **Abort / Stop Control** | Mengonsumsi Server-Sent Events (SSE) dengan `AbortController` (`Stop.svelte`) untuk menghentikan respon seketika. | `api.sendMessage()` menunggu panggilan HTTP selesai tanpa kemampuan pembatalan. | 🚨 **CRITICAL BUG** | Pengguna tidak dapat menghentikan model yang mengalami loop atau teks terlalu panjang, menghamburkan kuota token. |
| **Upload Dokumen & Gambar** | Drag-and-drop zone (`dropzone.svelte.ts`), paste clipboard gambar/file, chips file (`FileGroup.svelte`). | **TIDAK ADA**. Hanya ada textarea polos. | 🚨 **CRITICAL DEFECT** | Fitur multimodal LLM (analisis berkas, PDF, gambar, spreadsheet) tidak dapat digunakan sama sekali dari UI. |
| **Modal File Besar** | `ConvertToFileModal.svelte` otomatis mendeteksi teks paste berukuran besar dan menawarkan mengubahnya jadi file. | **TIDAK ADA**. Paste teks jutaan karakter membuat browser freeze. | ⚠️ **HIGH BUG** | Kerentanan browser macet saat menempelkan kode atau log server raksasa. |
| **Rekaman Audio** | `RecordAudioModal.svelte` dengan Web Audio API perekaman suara. | **TIDAK ADA**. | 🟡 **MEDIUM GAP** | Tidak ada input suara langsung (*speech-to-text*). |
| **Pemilihan Model Dinamis** | `ModelSelector.svelte` dengan pencarian model, deskripsi provider, dan konteks window. | Badge statis yang membaca nama model sesi saat dibuat tanpa dropdown pemilihan. | ⚠️ **HIGH GAP** | Pengguna tidak dapat berganti model secara instan sebelum mengirimkan pesan. |
| **Pemilihan Mode Chat** | `ModeSelector.svelte` (Normal Chat, Web Search, Deep Research, Media Generation). | **TIDAK ADA**. | ⚠️ **HIGH GAP** | Tidak dapat mengaktifkan grounding web search atau mode analitik khusus. |
| **Auto-Resize Textarea** | Membesar dan mengecil otomatis sesuai jumlah baris teks input. | Ketinggian kaku dengan scrollbar vertikal terbatas. | 🟡 **MEDIUM UX** | Pengalaman mengetik prompt panjang kurang leluasa. |

---

## 📂 Matrix 4: Manajemen Sesi & Sidebar (Session Lifecycle)

| Fitur Sidebar | Kemampuan Llumen (`sidebar/`) | Status Lumis UI | Klasifikasi Defek | Dampak Operasional & Kesenjangan |
| :--- | :--- | :--- | :---: | :--- |
| **Pencarian Sesi** | Input filter pencarian realtime berdasarkan kata kunci judul obrolan. | **TIDAK ADA**. | ⚠️ **HIGH DEFECT** | Pengguna yang memiliki puluhan sesi kesulitan menemukan obrolan terdahulu. |
| **Ubah Nama (Rename)** | `Entry.svelte` menyediakan menu ubah judul sesi secara langsung. | **TIDAK ADA**. Judul sesi terkunci permanen pada nama awal (`Chat 1`). | ⚠️ **HIGH DEFECT** | Pengguna tidak dapat mengorganisir riwayat chat. |
| **Hapus Sesi (Delete)** | Tombol hapus dengan modal konfirmasi aman (`CheckDelete.svelte`). | **TIDAK ADA**. Tidak ada tombol hapus sama sekali di antarmuka! | 🚨 **CRITICAL BUG** | Pengguna tidak dapat menghapus chat sampah atau obrolan yang memuat data rahasia. |
| **Sematkan Sesi (Pin)** | Sesi penting dapat di-pin ke bagian atas daftar. | Backend mendukung `is_pinned`, namun UI sama sekali tidak menyediakan kontrol ini! | ⚠️ **HIGH GAP** | Fitur basis data menganggur dan tidak terekspos ke antarmuka. |
| **Ekspor Percakapan** | Ekspor chat ke berkas Markdown atau JSON. | **TIDAK ADA**. | 🟡 **MEDIUM GAP** | Portabilitas data percakapan nol. |
| **Gestur Sentuh Mobile** | `gesture.ts` mendeteksi usapan jari (*touch swipe*) untuk menutup/membuka sidebar secara mulus. | Hanya tombol hamburger klik biasa. | 🟡 **MEDIUM UX** | Navigasi mobile terasa kaku tanpa dukungan gestur swipe natural. |

---

## ⚙️ Matrix 5: Preferensi, Pengaturan & Tata Kelola (Settings & Admin)

| Fitur Pengaturan | Kemampuan Llumen (`setting/`) | Status Lumis UI | Klasifikasi Defek | Dampak Operasional & Kesenjangan |
| :--- | :--- | :--- | :---: | :--- |
| **Modal Pengaturan** | Dialog multi-tab terstruktur (`Account`, `Admin`, `Openrouter`). | **TIDAK ADA**. Sama sekali tidak ada modal pengaturan! | 🚨 **CRITICAL DEFECT** | Pengguna tidak dapat menyesuaikan konfigurasi apa pun. |
| **Sistem Tema** | Pilihan tema (Llumen, Dracula, Vitesse) + Dark/Light toggle + Background pattern toggle. | Tema gelap kaku tunggal tanpa opsi kustomisasi atau mode terang (*light mode*). | ⚠️ **HIGH DEFECT** | Aksesibilitas visual rendah untuk pengguna di ruangan terang. |
| **Penggantian Sandi** | `PasswordSection.svelte` memvalidasi sandi lama dan sandi baru. | **TIDAK ADA**. | ⚠️ **HIGH DEFECT** | Akun pengguna lokal tidak dapat mengganti kredensial mandiri. |
| **Kelola Pengguna (Admin)**| `UserGrid.svelte` & `Admin.svelte` untuk membuat akun baru, melihat daftar pengguna, dan hak akses. | **TIDAK ADA**. | ⚠️ **HIGH ENTERPRISE GAP** | Admin harus mengeksekusi perintah SQL manual untuk manajemen user. |
| **Kunci API Kustom (BYOK)**| `Openrouter.svelte` memungkinkan pengguna memasukkan API key pribadi dan kustom base URL. | Terkunci pada variabel lingkungan backend server. | 🟡 **MEDIUM GAP** | Fitur multi-tenant BYOK enterprise tidak dapat digunakan oleh unit kerja. |

---

## 🎨 Matrix 6: Standar Desain shadcn-svelte & Higienitas Kode

| Dimensi Higienitas | Standar Baku Oktober 2026 | Evaluasi Lumis UI | Status Kepatuhan | Rekomendasi Refactoring |
| :--- | :--- | :--- | :---: | :--- |
| **Ukuran Berkas** | Maksimal 350 baris per file (`< 350 lines`). | `App.svelte` mencapai 309 baris (hampir menembus batas keras dalam 1 file). | 🟡 **RISK** | Pecah menjadi komponen atomik di `lib/components/` (`sidebar/`, `message/`, `input/`, `settings/`). |
| **shadcn-svelte Registry** | Menggunakan bits-ui + tailwind-variants standar (`Dialog`, `Tabs`, `Collapsible`, `Tooltip`). | Hanya memiliki Button, Badge, Textarea dasar. | ⚠️ **HIGH DEFECT** | Adopsi registry resmi shadcn-svelte untuk Dialog, Tabs, Accordion, DropdownMenu. |
| **Zero-Emoticon Policy** | 0 karakter emotikon Unicode; 100% ikon vektor `lucide-svelte`. | Patuh (menggunakan Lucide icons). | ✅ **PASS** | Pertahankan standar ini saat mengimplementasikan fitur-fitur baru. |
| **100% Google Sans** | Tipografi Google Sans, Google Sans Text, Google Sans Code seragam. | Patuh (terpasang di `index.html` dan `app.css`). | ✅ **PASS** | Pastikan seluruh komponen baru mewarisi `font-sans` dan `font-mono`. |
| **Word-Wrap Aware** | Wadah pesan wajib `[overflow-wrap:anywhere]` dan `break-words`. | Patuh pada container utama, namun belum ada parser Markdown untuk inline-code panjang. | ⚠️ **PARTIAL** | Pastikan renderer Code dan Table juga menerapkan overflow protection. |
| **Animasi 120Hz/144Hz** | HANYA menggunakan GPU compositor (`transform`, `opacity`) dengan kurva 120fps. | Patuh pada mobile drawer backdrop. | ✅ **PASS** | Terapkan pada collapsible accordion dan modal dialog baru. |
