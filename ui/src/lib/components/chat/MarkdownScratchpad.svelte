<script lang="ts">
  import MarkdownRenderer from "../markdown/MarkdownRenderer.svelte";
  import FileEdit from "lucide-svelte/icons/file-edit";
  import Eye from "lucide-svelte/icons/eye";
  import ArrowLeft from "lucide-svelte/icons/arrow-left";
  import { Button } from "$lib/components/ui/button";
  import { router } from "$lib/router.svelte";

  let markdownSource = $state(`# Lumis Markdown & LaTeX Scratchpad

Selamat datang di preview mandiri Markdown & KaTeX.

## Formula Matematika
Rumus kuadratik:
$$x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}$$

Rumus energi Einstein: $E = mc^2$.

## Blok Kode & Sintaksis
\`\`\`rust
fn calculate_entropy(data: &[u8]) -> f64 {
    // Kecepatan streaming SIMD
    42.0
}
\`\`\`

## Tabel Terformat
| Parameter | Tipe Data | Keterangan |
| :--- | :--- | :--- |
| \`session_id\` | UUID v4 | Pengidentifikasi unik sesi |
| \`latency_ms\` | u64 | Latensi inferensi milidetik |
`);
</script>

<div class="flex h-screen w-screen flex-col bg-background text-foreground font-sans select-none">
  <!-- Top Navigation -->
  <header class="flex h-14 items-center justify-between border-b border-border px-4 shrink-0 bg-card">
    <div class="flex items-center gap-3">
      <Button variant="ghost" size="icon" onclick={() => router.navigate("/chat/new")}>
        <ArrowLeft class="h-4 w-4" />
      </Button>
      <div class="flex items-center gap-2">
        <FileEdit class="h-4 w-4 text-cyan-400" />
        <span class="font-bold text-sm">Markdown Scratchpad</span>
      </div>
    </div>
  </header>

  <!-- Split View Area -->
  <div class="flex-1 grid grid-cols-1 md:grid-cols-2 overflow-hidden">
    <!-- Editor Pane -->
    <div class="flex flex-col border-b md:border-b-0 md:border-r border-border p-4 bg-muted/10">
      <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider mb-2 flex items-center gap-1.5">
        <FileEdit class="h-3.5 w-3.5" />
        <span>Editor Markdown Mentah</span>
      </div>
      <textarea
        bind:value={markdownSource}
        class="flex-1 w-full resize-none rounded-xl border border-border bg-card p-4 font-mono text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-cyan-500 leading-relaxed"
        placeholder="Ketik teks Markdown di sini..."
      ></textarea>
    </div>

    <!-- Live Preview Pane -->
    <div class="flex flex-col p-4 overflow-y-auto smooth-momentum-scroll bg-background">
      <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider mb-2 flex items-center gap-1.5">
        <Eye class="h-3.5 w-3.5 text-cyan-400" />
        <span>Pratinjau Terformat (Live AST)</span>
      </div>
      <div class="flex-1 rounded-xl border border-border/80 bg-card/60 p-6 shadow-xs">
        <MarkdownRenderer content={markdownSource} />
      </div>
    </div>
  </div>
</div>
