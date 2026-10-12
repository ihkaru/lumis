---
name: ui-design-hygiene
description: High-precision static code analysis and AST inspector for enforcing enterprise UI design hygiene, full shadcn-svelte standard component compliance, 120Hz/144Hz native refresh rate GPU-composited animations, zero emoticons (lucide-svelte vector icons), 100% Google Sans typography, word-wrap awareness, and mobile-first responsiveness per October 2026 standards.
---

# 🎨 UI Design Hygiene & shadcn-svelte Native Engine Inspector

Skill ini menyediakan prosedur dan engine analisis statis deterministik (`detect-ui-hygiene`) berbasis **AST Code-Graph Inspection** untuk menegakkan standardisasi desain enterprise modern pada frontend:
1. **Full shadcn-svelte Standard (https://www.shadcn-svelte.com/)**:
   - **Hemat Token & Zero-Reinvent**: Dilarang membangun komponen dasar dari nol (*from scratch*). Gunakan registry atomik `src/lib/components/ui/` (`Button`, `Badge`, `Textarea`, `Separator`, `Card`, `ScrollArea`) berbasis `bits-ui` dan `tailwind-variants`.
   - **Utilitas `cn()` Standar**: Seluruh penggabungan kelas dinamis wajib melewati `cn()` (`clsx` + `tailwind-merge`).
   - **Ikonografi `lucide-svelte`**: Menggunakan paket vektor resmi `lucide-svelte` (tanpa emotikon Unicode).
2. **Animasi Setinggi Refresh Rate Layar Asli (60Hz / 90Hz / 120Hz / 144Hz+)**:
   - **Compositor-Only Invariant**: Animasi HANYA boleh memanipulasi properti GPU compositor (`transform` dan `opacity`). Dilarang menganimasikan geometri layout (`width`, `height`, `left`, `right`, `top`, `bottom`, `margin`, `padding`).
   - **Hardware Layer Promotion**: Wajib memanfaatkan `transform: translate3d(0, 0, 0)`, `will-change: transform`, dan `backface-visibility: hidden`.
   - **Kurva Easing 120fps**: `cubic-bezier(0.16, 1, 0.3, 1)` untuk akselerasi dan deselerasi sehalus refresh rate layar fisik.
3. **Zero Emoticon Policy**: Larangan total karakter emotikon/emoji Unicode di jalur UI; wajib menggunakan ikon vektor `lucide-svelte` / SVG murni.
4. **100% Google Sans Typography**: Tipografi seragam (`Google Sans`, `Google Sans Text`, `Google Sans Code`) tanpa fallback generic yang mendahului.
5. **Word-Wrap & Overflow Awareness**: Pencegahan kebocoran tata letak horizontal (*horizontal scroll blowout*) pada pesan percakapan dan string token panjang (`[overflow-wrap:anywhere]`).
6. **Mobile-First Responsive Layout**: Drawer navigasi adaptif, backdrop penutup sentuh (*tap-away*), dan target sentuh minimal $\ge 44\text{px}$.

---

## 🎯 Kapan Menggunakan Skill Ini

1. **Sebelum Merilis / Men-deploy Kode Frontend (`ui/`)**:
   - Memastikan bahwa UI bersih dari emotikon, aman dari overflow teks, responsif di smartphone, dan animasi berjalan mulus pada 120Hz/144Hz.
2. **Saat Menjalankan CI/CD & Governance Audit**:
   - Mengintegrasikan `detect-ui-hygiene` ke dalam hook pra-komit atau script tata kelola (`scripts/governance-check.sh`).
3. **Saat Menambah Komponen UI Baru (`.svelte`, `.ts`, `.html`, `.css`)**:
   - Memanfaatkan komponen registry shadcn-svelte yang sudah ada di `src/lib/components/ui/` untuk menghemat token dan menjaga konsistensi.

---

## 🛠️ CLI Tool Global: `detect-ui-hygiene`

Tool ini telah terpasang secara global di `/usr/local/bin/detect-ui-hygiene`.

```bash
# Pindai direktori ui/ proyek
detect-ui-hygiene ui

# Mode JSON (Otomasi Agen & CI/CD)
detect-ui-hygiene ui --json
```

---

## 📐 Taksonomi Pelanggaran Desain Hygiene (Standar Oktober 2026)

| Tingkat | Kategori Rule | Contoh Deteksi | Aksi Perbaikan |
|---|---|---|---|
| 🚨 **CRITICAL** | `RULE_ZERO_EMOTICON` | `<span>⚡</span>`, `🎉`, `🤖` di komponen UI | Ganti dengan komponen `lucide-svelte` (e.g. `Zap`, `Sparkles`, `Bot`). |
| 🚨 **CRITICAL** | `RULE_GOOGLE_SANS_ENFORCED` | `font-family: -apple-system...` tanpa `Google Sans` | Sertakan Google Sans di `index.html`, `app.css`, dan `tailwind.config.js`. |
| 🚨 **CRITICAL** | `RULE_SHADCN_COMPLIANCE` | Tombol/lencana custom raw tanpa `cn()` atau konfigurasi `components.json` hilang | Gunakan komponen `shadcn-svelte` dan helper `cn()`. |
| ⚠️ **HIGH** | `RULE_HIGH_REFRESH_RATE_ANIMATION` | Menganimasikan `width`/`left` atau ketiadaan `will-change`/GPU layer pada drawer | Ubah ke `transform: translate3d` dengan `will-change: transform`. |
| ⚠️ **HIGH** | `RULE_WORD_WRAP_AWARE` | Bubble chat tanpa `break-words` atau `[overflow-wrap:anywhere]` | Tambahkan kelas Tailwind `break-words whitespace-pre-wrap [overflow-wrap:anywhere]`. |
| ⚠️ **HIGH** | `RULE_MOBILE_RESPONSIVE` | Sidebar statis `w-72` tanpa drawer mobile, atau touch targets `< 44px` | Terapkan drawer responsif (`md:` breakpoint), backdrop dismiss, dan touch target $\ge 44\text{px}$. |

---

## 🔬 Integrasi Code-Graph AST

Tool `detect-ui-hygiene` terintegrasi dengan AST engine `code-graph`:
- `code-graph overview ui/src`: Memverifikasi struktur modul dan mendeteksi dependensi UI yang tidak valid.
- AST Token Scanner: Memindai seluruh token harfiah string dan template literal untuk memblokir rentang Unicode emoji `U+1F300..U+1FAFF`, `U+2600..U+27BF`.
