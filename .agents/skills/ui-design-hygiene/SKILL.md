---
name: ui-design-hygiene
description: High-precision static code analysis and AST inspector for enforcing enterprise UI design hygiene, zero emoticons (pure vector SVG icons), 100% Google Sans typography, word-wrap awareness, and mobile-first responsiveness per October 2026 standards.
---

# 🎨 UI Design Hygiene & Governance Inspector

Skill ini menyediakan prosedur dan engine analisis statis deterministik (`detect-ui-hygiene`) berbasis **AST Code-Graph Inspection** untuk menegakkan standardisasi desain enterprise modern pada frontend:
1. **Zero Emoticon Policy**: Larangan total karakter emotikon/emoji Unicode di jalur UI; wajib menggunakan ikon vektor `<svg>` murni.
2. **100% Google Sans Typography**: Tipografi seragam (`Google Sans`, `Google Sans Text`, `Google Sans Code`) tanpa fallback generic yang mendahului.
3. **Word-Wrap & Overflow Awareness**: Pencegahan kebocoran tata letak horizontal (*horizontal scroll blowout*) pada pesan percakapan dan string panjang.
4. **Mobile-First Responsive Layout**: Drawer navigasi adaptif, backdrop penutup sentuh (*tap-away*), dan target sentuh minimal $\ge 44\text{px}$.

---

## 🎯 Kapan Menggunakan Skill Ini

1. **Sebelum Merilis / Men-deploy Kode Frontend (`ui/`)**:
   - Memastikan bahwa UI bersih dari emotikon, aman dari overflow teks, dan responsif di smartphone.
2. **Saat Menjalankan CI/CD & Governance Audit**:
   - Mengintegrasikan `detect-ui-hygiene` ke dalam hook pra-komit atau script tata kelola (`scripts/governance-check.sh`).
3. **Saat Menambah Komponen UI Baru (`.svelte`, `.ts`, `.html`, `.css`)**:
   - Memastikan ikon yang ditambahkan berupa SVG terstruktur, bukan emoji atau simbol mentah teks.

---

## 🛠️ CLI Tool Global: `detect-ui-hygiene`

Tool ini telah terpasang secara global di `/usr/local/bin/detect-ui-hygiene`.

### 1. Perintah Standar (Interaktif / Terminal)
```bash
# Pindai direktori ui/ proyek
detect-ui-hygiene ui

# Pindai path spesifik
detect-ui-hygiene /path/to/ui
```

### 2. Mode JSON (Otomasi Agen & CI/CD)
```bash
detect-ui-hygiene ui --json
```

---

## 📐 Taksonomi Pelanggaran Desain Hygiene (Standar Oktober 2026)

| Tingkat | Kategori Rule | Contoh Deteksi | Aksi Perbaikan |
|---|---|---|---|
| 🚨 **CRITICAL** | `RULE_ZERO_EMOTICON` | `<span>⚡</span>`, `🎉`, `🤖` di komponen UI | Ganti dengan elemen vektor `<svg viewBox="0 0 24 24">` atau komponen ikon. |
| 🚨 **CRITICAL** | `RULE_GOOGLE_SANS_ENFORCED` | `font-family: -apple-system...` tanpa `Google Sans`, atau tidak ada link CDN di `index.html` | Sertakan Google Sans di `index.html`, `app.css`, dan `tailwind.config.js`. |
| ⚠️ **HIGH** | `RULE_WORD_WRAP_AWARE` | Bubble chat tanpa `break-words` atau `[overflow-wrap:anywhere]` | Tambahkan kelas Tailwind `break-words whitespace-pre-wrap [overflow-wrap:anywhere]`. |
| ⚠️ **HIGH** | `RULE_MOBILE_RESPONSIVE` | Sidebar statis `w-72` tanpa drawer mobile, atau touch targets `< 44px` | Terapkan drawer responsif (`md:` breakpoint), backdrop dismiss, dan touch target $\ge 44\text{px}$. |

---

## 🔬 Integrasi Code-Graph AST

Tool `detect-ui-hygiene` terintegrasi dengan AST engine `code-graph`:
- `code-graph overview ui/src`: Memverifikasi struktur modul dan mendeteksi dependensi UI yang tidak valid.
- AST Token Scanner: Memindai seluruh token harfiah string dan template literal untuk memblokir rentang Unicode emoji `U+1F300..U+1FAFF`, `U+2600..U+27BF`.
