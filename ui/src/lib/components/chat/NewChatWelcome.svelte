<script lang="ts">
  import Sparkles from "lucide-svelte/icons/sparkles";
  import Code2 from "lucide-svelte/icons/code-2";
  import FileSearch from "lucide-svelte/icons/file-search";
  import Cpu from "lucide-svelte/icons/cpu";
  import BarChart3 from "lucide-svelte/icons/bar-chart-3";

  interface Props {
    brandTitle?: string;
    onSelectPrompt?: (promptText: string) => void;
  }

  let { brandTitle = "Lumis Enterprise AI", onSelectPrompt }: Props = $props();

  const starterPrompts = [
    {
      icon: Code2,
      title: "Arsitektur Microservice",
      desc: "Rancang REST/gRPC API scalable dengan Rust Axum dan SQLite WAL.",
      prompt: "Rancang arsitektur REST API berkinerja tinggi menggunakan Rust Axum dan database SQLite WAL.",
    },
    {
      icon: FileSearch,
      title: "Analisis Data Sensitif & DLP",
      desc: "Evaluasi dokumen dan dataset organisasi dengan kepatuhan privasi dan DLP.",
      prompt: "Jelaskan langkah audit DLP dan kepatuhan privasi untuk data sensitif perusahaan.",
    },
    {
      icon: Cpu,
      title: "Optimasi Query SQLx",
      desc: "Tulis keyset cursor pagination O(log N) untuk tabel percakapan jutaan baris.",
      prompt: "Buat query SQL keyset pagination dengan indeks komposit (created_at, id) untuk efisiensi O(log N).",
    },
    {
      icon: BarChart3,
      title: "Visualisasi Grafik & Mermaid",
      desc: "Buat diagram alur pemrosesan data end-to-end dalam format Mermaid.",
      prompt: "Buatkan diagram arsitektur Mermaid flowchart TD untuk alur ingestion data streaming.",
    },
  ];
</script>

<div class="flex flex-col items-center justify-center max-w-3xl mx-auto px-4 py-8 md:py-16 text-center select-none font-sans">
  <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary/10 border border-primary/25 text-primary text-xs font-mono mb-4">
    <Sparkles class="h-3.5 w-3.5" />
    <span>Enterprise AI Model Orchestration</span>
  </div>

  <h1 class="text-3xl md:text-5xl font-extrabold tracking-tight bg-gradient-to-r from-primary via-primary/90 to-primary/75 bg-clip-text text-transparent pb-2">
    {brandTitle}
  </h1>
  <p class="mt-2 text-sm md:text-base text-muted-foreground max-w-xl">
    Mulai percakapan baru dengan model reasoning terdepan atau pilih rekomendasi prompt di bawah:
  </p>

  <div class="mt-8 grid grid-cols-1 md:grid-cols-2 gap-3 w-full text-left">
    {#each starterPrompts as item}
      <button
        type="button"
        onclick={() => onSelectPrompt?.(item.prompt)}
        class="group flex flex-col p-4 rounded-xl border border-border/80 bg-card/60 hover:bg-card hover:border-primary/40 transition-all duration-150 shadow-sm cursor-pointer text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
      >
        <div class="flex items-center gap-2.5 mb-1.5">
          <div class="p-1.5 rounded-lg bg-primary/10 text-primary border border-primary/25 group-hover:scale-105 transition-transform">
            <item.icon class="h-4 w-4" />
          </div>
          <span class="font-semibold text-sm text-foreground group-hover:text-primary transition-colors">
            {item.title}
          </span>
        </div>
        <p class="text-xs text-muted-foreground leading-relaxed">
          {item.desc}
        </p>
      </button>
    {/each}
  </div>
</div>
