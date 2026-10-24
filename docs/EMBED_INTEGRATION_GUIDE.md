# 🧩 Panduan Integrasi & Embed Lumis Chat ke Existing SPA

Dokumen ini adalah panduan komprehensif bagi tim pengembang untuk mengintegrasikan atau meng-embed antarmuka dan mekanisme percakapan cerdas **Lumis** ke dalam aplikasi Single Page Application (SPA) yang sudah ada (React, Vue, Angular, Svelte, atau Vanilla HTML/JS) secara aman, modular, dan mulus (*zero context switching*).

---

## 🎯 4 Pendekatan Integrasi (Pilih Sesuai Kebutuhan)

```mermaid
flowchart TD
    A["Kebutuhan Integrasi"] --> B{"Tingkat Kedalaman & Isolasi"}
    B -->|"Zero CSS Conflict & Drop-in Cepat"| C["Metode 1: Drop-In Script Loader (lumis-embed.js)"]
    B -->|"Sandbox Keamanan Mutlak / Multi-Domain"| D["Metode 2: Scoped Iframe + PostMessage RPC"]
    B -->|"Native Web Standards / Tanpa Iframe"| E["Metode 3: Svelte 5 Custom Element (<lumis-chat>)"]
    B -->|"Kontrol DOM & UI Kustom Mendalam di React/Svelte"| F["Metode 4: Headless Core SDK (@lumis/sdk-core)"]
```

---

## 🚀 Metode 1: 1-Line Drop-In Script Loader (`lumis-embed.js`)
*Pilihan tercepat untuk menyematkan floating chat widget di website atau portal enterprise internal.*

Cukup tempelkan baris script berikut sebelum penutup tag `</body>`:

```html
<script
  src="https://lumis.company.com/lumis-embed.js"
  data-server-url="https://lumis.company.com"
  data-token="EMBED_TOKEN_DARI_BACKEND_ANDA"
  data-position="bottom-right"
  data-title="Lumis Enterprise Assistant"
  data-theme="auto"
  async>
</script>
```

Atau inisialisasi secara dinamis via JavaScript:

```javascript
import { initLumisWidget } from "@lumis/embed";

const widget = initLumisWidget({
  serverUrl: "https://lumis.company.com",
  embedToken: await fetchEmbedTokenFromBackend(),
  position: "bottom-right", // atau "bottom-left"
  title: "BPS AI Assistant",
  contextData: {
    active_module: "Kepegawaian",
    current_employee_id: "199401122018011001"
  }
});

// Kontrol Programatik:
widget.open();
widget.sendPrompt("Bantu saya rekap data bulan ini");
```

---

## 🔒 Metode 2: Scoped Iframe + PostMessage RPC
*Pilihan terbaik jika aplikasi host memiliki aturan CSS yang agresif atau kebijakan keamanan CSP ketat.*

### 1. Pasang Iframe di Halaman Host
```html
<iframe
  id="lumis-chat-frame"
  src="https://lumis.company.com/embed/chat?token=EMBED_TOKEN&origin=https://my-spa.corp"
  style="width: 100%; height: 600px; border: none; border-radius: 12px;"
  allow="clipboard-write">
</iframe>
```

### 2. Komunikasi Dwiarah PostMessage (RFC 6454 Safe)
```javascript
const frame = document.getElementById("lumis-chat-frame");
const LUMIS_ORIGIN = "https://lumis.company.com";

// Mendengarkan event dari Lumis Widget
window.addEventListener("message", (event) => {
  if (event.origin !== LUMIS_ORIGIN) return; // Validasi origin wajib!
  const { source, event: evtName, payload, correlation_id } = event.data || {};
  if (source !== "lumis-widget") return;

  if (evtName === "lumis:ready") {
    // Widget siap menerima konteks
    frame.contentWindow.postMessage({
      source: "host-app",
      event: "lumis:set_context",
      payload: { active_table_filter: "rejected" }
    }, LUMIS_ORIGIN);
  }
});
```

---

## ⚛️ Metode 3: Integrasi Native di React SPA
*Menggunakan Headless SDK `@lumis/sdk-core` untuk integrasi reaktif penuh dengan state React.*

```bash
bun add @lumis/sdk-core
```

```tsx
import React, { useEffect, useState } from "react";
import { HttpSseTransport, createLumisChat } from "@lumis/sdk-core";

export function EmbeddedAssistant({ userToken, documentId }: { userToken: string; documentId: string }) {
  const [engine] = useState(() => {
    const transport = new HttpSseTransport({
      baseUrl: "https://lumis.company.com",
      embedToken: userToken
    });
    return createLumisChat(transport, {
      contextData: { document_id: documentId }
    });
  });

  useEffect(() => {
    engine.initStream();
    return () => engine.disconnectStream();
  }, [engine]);

  return (
    <div className="flex flex-col h-[500px] border rounded-xl overflow-hidden">
      <div className="p-3 bg-slate-100 font-semibold text-sm">Lumis In-App Assistant</div>
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {engine.messages.map((msg) => (
          <div key={msg.id} className={msg.role === "user" ? "text-right" : "text-left"}>
            <span className="inline-block p-2 rounded-lg bg-slate-200 text-sm">{msg.content}</span>
          </div>
        ))}
      </div>
      <form onSubmit={(e) => {
        e.preventDefault();
        const input = (e.currentTarget.elements.namedItem("prompt") as HTMLInputElement);
        if (input.value) { engine.sendMessage(input.value); input.value = ""; }
      }} className="p-2 border-t flex gap-2">
        <input name="prompt" className="flex-1 border rounded px-3 py-1.5 text-sm" placeholder="Ketik pesan..." />
        <button type="submit" disabled={engine.isGenerating} className="bg-blue-600 text-white px-4 py-1.5 rounded text-sm">Kirim</button>
      </form>
    </div>
  );
}
```

---

## 🔑 Backend: Cara Menghasilkan Ephemeral Embed Token

Host backend Anda menghasilkan token aman jangka pendek (15–30 menit) menggunakan endpoint `POST /api/auth/embed-token` di Lumis:

```bash
curl -X POST https://lumis.company.com/api/auth/embed-token \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer MASTER_API_KEY_ANDA" \
  -d '{
    "external_user_id": "usr_karyawan_1982",
    "username": "budi.santoso",
    "tenant_id": "bps",
    "roles": ["member"],
    "email": "budi@bps.go.id",
    "ttl_seconds": 1800
  }'
```

**Respons JSON:**
```json
{
  "token": "lumis_embed_v1.eyJzdWIiOiJ1c3Jfa2FyeWF3YW5fMTk4MiIsImV4cCI6MTcyOTc4NDk5Mn0....signature",
  "user_id": "usr_karyawan_1982",
  "expires_at": 1729784992
}
```

Token ini diserahkan ke browser frontend Host SPA untuk mengotentikasi obrolan Lumis tanpa membagikan kredensial login atau cookie utama.

---

## 🛠️ Agentic In-App Tools (Model AI Memanggil Aksi di Aplikasi Host)

Anda dapat mendaftarkan fungsi JavaScript di aplikasi host yang dapat dipanggil langsung oleh model AI:

```javascript
widget.registerTool({
  name: "navigate_dashboard",
  description: "Pindah ke halaman tertentu pada dashboard aplikasi",
  parameters: {
    route: "string (contoh: /laporan atau /profil)"
  },
  execute: async ({ route }) => {
    // Navigasi router host aplikasi Anda (misal: react-router / vue-router)
    myAppRouter.push(route);
    return { success: true, message: `Berhasil diarahkan ke ${route}` };
  }
});
```

---

## 🛡️ Praktik Terbaik Keamanan & Kepatuhan CISO

1. **Konfigurasi CORS**: Tentukan domain resmi host pada environment variable Lumis Server:
   `CORS_ALLOWED_ORIGINS="https://portal.bps.go.id,https://admin.bps.go.id"`
2. **Validasi Origin**: Selalu periksa `event.origin` saat menangani pesan `postMessage`.
3. **Masa Berlaku Token Pendek**: Terapkan masa berlaku token embed antara 15 hingga 60 menit.
4. **Isolasi Konteks Data**: Manfaatkan `contextData` (EIP Content Enricher) untuk menyuntikkan ID entitas aktif tanpa membocorkan token rahasia ke prompt pengguna.
