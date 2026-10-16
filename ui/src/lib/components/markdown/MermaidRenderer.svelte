<script lang="ts">
  import GitGraph from "lucide-svelte/icons/git-graph";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import ZoomIn from "lucide-svelte/icons/zoom-in";
  import ZoomOut from "lucide-svelte/icons/zoom-out";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Code2 from "lucide-svelte/icons/code-2";
  import Eye from "lucide-svelte/icons/eye";
  import mermaid from "mermaid";

  interface Props {
    code: string;
  }

  let { code = "" }: Props = $props();

  let svgHtml = $state("");
  let renderError = $state<string | null>(null);
  let isCopied = $state(false);
  let showRawCode = $state(false);
  let zoomLevel = $state(100);

  let isInitialized = false;

  function ensureInitialized() {
    if (isInitialized || typeof window === "undefined") return;
    try {
      mermaid.initialize({
        startOnLoad: false,
        theme: "dark",
        securityLevel: "loose",
        suppressErrorRendering: true,
        fontFamily: "Google Sans Text, system-ui, sans-serif",
        themeVariables: {
          darkMode: true,
          background: "transparent",
          primaryColor: "#0284c7",
          primaryTextColor: "#f1f5f9",
          primaryBorderColor: "#38bdf8",
          lineColor: "#64748b",
          secondaryColor: "#1e293b",
          tertiaryColor: "#0f172a",
        },
      });
      isInitialized = true;
    } catch {}
  }

  async function renderChart() {
    if (typeof window === "undefined" || !code.trim()) return;
    ensureInitialized();
    try {
      renderError = null;
      const cleanCode = code.trim();
      const uniqueId = `mmd_${Math.random().toString(36).slice(2, 9)}`;
      const { svg } = await mermaid.render(uniqueId, cleanCode);
      svgHtml = svg;
    } catch (err: any) {
      renderError = err?.message || "Diagram sedang diproses...";
    }
  }

  $effect(() => {
    if (code) {
      renderChart();
    }
  });

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(code);
      isCopied = true;
      setTimeout(() => (isCopied = false), 1800);
    } catch {}
  }

  function zoomIn() {
    if (zoomLevel < 160) zoomLevel = Math.min(160, zoomLevel + 15);
  }

  function zoomOut() {
    if (zoomLevel > 40) zoomLevel = Math.max(40, zoomLevel - 15);
  }

  function resetZoom() {
    zoomLevel = 100;
  }
</script>

<div class="my-1 overflow-hidden rounded-md border border-border/25 dark:border-white/10 bg-slate-950/85 font-sans text-xs shadow-xs">
  <!-- Header Bar -->
  <div class="flex items-center justify-between border-b border-border/15 bg-slate-900/60 px-2.5 py-0.5 select-none">
    <div class="flex items-center gap-1.5 text-primary/90">
      <GitGraph class="h-3 w-3" />
      <span class="font-medium text-[10px] tracking-wider uppercase font-mono text-slate-300">
        Mermaid Diagram
      </span>
      {#if renderError && !showRawCode}
        <span class="text-[9px] text-amber-400/80 bg-amber-400/10 px-1 py-0.2 rounded font-mono">
          sintaks raw
        </span>
      {/if}
    </div>

    <div class="flex items-center gap-0.5">
      <!-- Toggle Code / Render -->
      {#if svgHtml && !renderError}
        <button
          type="button"
          onclick={() => (showRawCode = !showRawCode)}
          class="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800/80 transition-colors cursor-pointer"
          title={showRawCode ? "Lihat Diagram" : "Lihat Sintaks"}
          aria-label={showRawCode ? "Lihat Diagram" : "Lihat Sintaks"}
        >
          {#if showRawCode}
            <Eye class="h-3 w-3" />
          {:else}
            <Code2 class="h-3 w-3" />
          {/if}
        </button>

        <!-- Zoom Controls -->
        <button
          type="button"
          onclick={zoomOut}
          disabled={zoomLevel <= 40}
          class="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800/80 disabled:opacity-30 transition-colors cursor-pointer"
          title="Zoom Out"
          aria-label="Zoom Out"
        >
          <ZoomOut class="h-3 w-3" />
        </button>
        <button
          type="button"
          onclick={zoomIn}
          disabled={zoomLevel >= 160}
          class="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800/80 disabled:opacity-30 transition-colors cursor-pointer"
          title="Zoom In"
          aria-label="Zoom In"
        >
          <ZoomIn class="h-3 w-3" />
        </button>

        {#if zoomLevel !== 100}
          <button
            type="button"
            onclick={resetZoom}
            class="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800/80 transition-colors cursor-pointer"
            title="Reset Zoom"
            aria-label="Reset Zoom"
          >
            <RotateCcw class="h-2.5 w-2.5" />
          </button>
        {/if}
        <div class="h-3 w-px bg-border/40 mx-0.5" aria-hidden="true"></div>
      {/if}

      <!-- Copy Button -->
      <button
        type="button"
        onclick={handleCopy}
        class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] text-slate-400 hover:text-primary hover:bg-slate-800/80 transition-colors cursor-pointer"
        title="Salin kode diagram"
        aria-label="Salin kode diagram"
      >
        {#if isCopied}
          <Check class="h-3 w-3 text-emerald-400" />
          <span class="text-emerald-400">Tersalin</span>
        {:else}
          <Copy class="h-3 w-3" />
          <span>Salin</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Content Container -->
  {#if svgHtml && !renderError && !showRawCode}
    <div class="mermaid-viewport w-full overflow-x-auto p-3 flex justify-center items-center bg-card/10 select-none">
      <div
        class="mermaid-svg-container transition-transform duration-150 inline-block"
        style="transform: scale({zoomLevel / 100}); transform-origin: center top;"
      >
        {@html svgHtml}
      </div>
    </div>
  {:else}
    <!-- Raw code view (fallback during streaming, error, or explicit toggle) -->
    <div class="px-2.5 py-1.5 bg-slate-950 overflow-x-auto text-slate-300 font-mono text-[11px] whitespace-pre leading-relaxed">
      <code>{code}</code>
    </div>
  {/if}
</div>

<style>
  :global(.mermaid-svg-container svg) {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 0 auto;
  }
</style>

