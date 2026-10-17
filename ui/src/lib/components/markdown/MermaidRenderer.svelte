<script lang="ts">
  import GitGraph from "lucide-svelte/icons/git-graph";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import Maximize2 from "lucide-svelte/icons/maximize-2";
  import X from "lucide-svelte/icons/x";
  import Code2 from "lucide-svelte/icons/code-2";
  import Eye from "lucide-svelte/icons/eye";
  import Contrast from "lucide-svelte/icons/contrast";
  import Sun from "lucide-svelte/icons/sun";
  import Moon from "lucide-svelte/icons/moon";
  import mermaid from "mermaid";
  import DiagramNavControls from "./DiagramNavControls.svelte";
  import { cleanMermaidSvg, getMermaidConfig } from "./mermaidConfig";

  interface Props {
    code: string;
  }

  let { code = "" }: Props = $props();

  let svgHtml = $state("");
  let renderError = $state<string | null>(null);
  let isCopied = $state(false);
  let showRawCode = $state(false);
  let isFullscreen = $state(false);
  let canvasContrast = $state<"default" | "light" | "deep">("default");

  // Pan & Zoom Reactive States
  let scale = $state(1);
  let panX = $state(0);
  let panY = $state(0);
  let isDragging = $state(false);
  let dragStartX = 0;
  let dragStartY = 0;
  let initialPanX = 0;
  let initialPanY = 0;

  let isDark = $state(
    typeof document !== "undefined"
      ? document.documentElement.classList.contains("dark")
      : true
  );

  let canvasClass = $derived.by(() => {
    if (canvasContrast === "light") return "bg-white text-slate-900 contrast-light";
    if (canvasContrast === "deep") return "bg-black text-slate-100 contrast-deep";
    return "bg-slate-950/85 text-slate-200";
  });

  function toggleContrast() {
    if (canvasContrast === "default") canvasContrast = "light";
    else if (canvasContrast === "light") canvasContrast = "deep";
    else canvasContrast = "default";
  }

  async function renderChart() {
    if (typeof window === "undefined" || !code.trim()) return;
    try {
      renderError = null;
      const cleanCode = code.trim();
      const uniqueId = `mmd_${Math.random().toString(36).slice(2, 9)}`;
      const effectiveDark = canvasContrast === "light" ? false : isDark;
      mermaid.initialize(getMermaidConfig(effectiveDark));
      const { svg } = await mermaid.render(uniqueId, cleanCode);
      svgHtml = cleanMermaidSvg(svg, canvasContrast === "light");
    } catch (err: any) {
      renderError = err?.message || "Diagram sedang diproses...";
    }
  }

  $effect(() => {
    if (code || canvasContrast !== undefined) {
      renderChart();
    }
  });

  $effect(() => {
    if (typeof document === "undefined") return;
    const observer = new MutationObserver(() => {
      const nextDark = document.documentElement.classList.contains("dark");
      if (nextDark !== isDark) {
        isDark = nextDark;
        renderChart();
      }
    });
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
    return () => observer.disconnect();
  });

  $effect(() => {
    if (typeof window === "undefined") return;
    function handleKey(e: KeyboardEvent) { if (e.key === "Escape" && isFullscreen) isFullscreen = false; }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  });

  async function handleCopy() {
    try { await navigator.clipboard.writeText(code); isCopied = true; setTimeout(() => (isCopied = false), 1800); } catch {}
  }

  function zoomBy(factor: number) { scale = Math.min(Math.max(scale * factor, 0.25), 4.5); }
  function panBy(dx: number, dy: number) { panX += dx; panY += dy; }
  function resetView() { scale = 1; panX = 0; panY = 0; }

  function handlePointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    isDragging = true; dragStartX = e.clientX; dragStartY = e.clientY;
    initialPanX = panX; initialPanY = panY;
    (e.currentTarget as HTMLElement)?.setPointerCapture?.(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isDragging) return;
    panX = initialPanX + (e.clientX - dragStartX);
    panY = initialPanY + (e.clientY - dragStartY);
  }

  function handlePointerUp(e: PointerEvent) {
    if (!isDragging) return;
    isDragging = false;
    try { (e.currentTarget as HTMLElement)?.releasePointerCapture?.(e.pointerId); } catch {}
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    zoomBy(e.deltaY < 0 ? 1.1 : 0.9);
  }
</script>

<!-- Inline Diagram Card -->
<div class="my-1.5 overflow-hidden rounded-md border border-border/25 dark:border-white/10 bg-slate-950 font-sans text-xs shadow-xs select-none">
  <!-- Header Bar -->
  <div class="flex items-center justify-between border-b border-border/15 bg-slate-900/60 px-2.5 py-1">
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
      {#if svgHtml && !renderError}
        <!-- Contrast Toggle Button -->
        <button
          type="button"
          onclick={toggleContrast}
          class="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors"
          title="Ubah Kontras Kanvas ({canvasContrast === 'default' ? 'Standar' : canvasContrast === 'light' ? 'Terang' : 'Hitam'})"
        >
          {#if canvasContrast === "light"}
            <Sun class="h-3 w-3 text-amber-400" />
          {:else if canvasContrast === "deep"}
            <Moon class="h-3 w-3 text-cyan-400" />
          {:else}
            <Contrast class="h-3 w-3" />
          {/if}
        </button>

        <!-- Toggle Code / Render -->
        <button
          type="button"
          onclick={() => (showRawCode = !showRawCode)}
          class="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors"
          title={showRawCode ? "Lihat Diagram" : "Lihat Sintaks"}
        >
          {#if showRawCode}
            <Eye class="h-3 w-3" />
          {:else}
            <Code2 class="h-3 w-3" />
          {/if}
        </button>

        <button
          type="button"
          onclick={() => (isFullscreen = !isFullscreen)}
          class="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors"
          title="Fullscreen Diagram"
        >
          <Maximize2 class="h-3 w-3" />
        </button>
      {/if}

      <!-- Copy Button -->
      <button
        type="button"
        onclick={handleCopy}
        class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] text-slate-400 hover:text-primary hover:bg-slate-800 transition-colors"
        title="Salin kode diagram"
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

  <!-- Content Viewport -->
  {#if svgHtml && !renderError && !showRawCode}
    <div
      class="relative w-full h-[360px] overflow-hidden {canvasClass} {isDragging ? 'cursor-grabbing' : 'cursor-grab'} touch-none transition-colors duration-200"
      onpointerdown={handlePointerDown}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      onpointercancel={handlePointerUp}
      onwheel={handleWheel}
      role="application"
      aria-label="Interactive Diagram Canvas"
    >
      <!-- Diagram SVG Container -->
      <div
        class="absolute inset-0 flex items-center justify-center pointer-events-none will-change-transform"
        style="transform: translate3d({panX}px, {panY}px, 0) scale({scale}); transform-origin: center center;"
      >
        <div class="mermaid-svg-container pointer-events-auto">
          {@html svgHtml}
        </div>
      </div>

      <!-- Navigation & D-Pad Overlay (Bottom Right) -->
      <div class="absolute bottom-2.5 right-2.5 z-10 pointer-events-auto">
        <DiagramNavControls {scale} step={60} onPan={panBy} onZoom={zoomBy} onReset={resetView} />
      </div>
    </div>
  {:else}
    <div class="px-2.5 py-2 bg-slate-950 overflow-x-auto text-slate-300 font-mono text-[11px] whitespace-pre leading-relaxed">
      <code>{code}</code>
    </div>
  {/if}
</div>

<!-- Fullscreen Canvas Modal -->
{#if isFullscreen && svgHtml}
  <div class="fixed inset-0 z-50 flex flex-col bg-slate-950/95 backdrop-blur-md p-3 sm:p-5">
    <div class="flex items-center justify-between border-b border-white/10 pb-2 mb-2 select-none">
      <div class="flex items-center gap-2">
        <GitGraph class="h-4 w-4 text-primary" />
        <span class="font-semibold text-sm text-slate-100">Mermaid Diagram Canvas - Fullscreen</span>
      </div>
      <div class="flex items-center gap-1.5">
        <button
          type="button"
          onclick={toggleContrast}
          class="p-1.5 rounded hover:bg-white/10 text-slate-300 hover:text-white transition-colors"
          title="Ubah Kontras Kanvas"
        >
          {#if canvasContrast === "light"}
            <Sun class="h-4 w-4 text-amber-400" />
          {:else if canvasContrast === "deep"}
            <Moon class="h-4 w-4 text-cyan-400" />
          {:else}
            <Contrast class="h-4 w-4" />
          {/if}
        </button>
        <button
          type="button"
          onclick={() => (isFullscreen = false)}
          class="p-1.5 rounded hover:bg-white/10 text-slate-300 hover:text-white transition-colors"
          title="Tutup Fullscreen (Esc)"
        >
          <X class="h-4 w-4" />
        </button>
      </div>
    </div>

    <!-- Fullscreen Interactive Canvas -->
    <div
      class="relative flex-1 w-full rounded-lg border border-white/10 {canvasClass} overflow-hidden shadow-2xl {isDragging ? 'cursor-grabbing' : 'cursor-grab'} touch-none transition-colors duration-200"
      onpointerdown={handlePointerDown}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      onpointercancel={handlePointerUp}
      onwheel={handleWheel}
      role="application"
      aria-label="Fullscreen Diagram Canvas"
    >
      <div
        class="absolute inset-0 flex items-center justify-center pointer-events-none will-change-transform"
        style="transform: translate3d({panX}px, {panY}px, 0) scale({scale}); transform-origin: center center;"
      >
        <div class="mermaid-svg-container pointer-events-auto">
          {@html svgHtml}
        </div>
      </div>

      <!-- Navigation & D-Pad in Fullscreen -->
      <div class="absolute bottom-4 right-4 z-10 pointer-events-auto">
        <DiagramNavControls {scale} step={100} onPan={panBy} onZoom={zoomBy} onReset={resetView} />
      </div>
    </div>
  </div>
{/if}

<style>
  :global(.mermaid-svg-container svg) { max-width: none !important; height: auto; display: block; margin: 0 auto; font-family: "Google Sans Text", system-ui, sans-serif !important; }
  :global(.mermaid-svg-container svg text) { font-family: "Google Sans Text", system-ui, sans-serif !important; }
  :global(.mermaid-svg-container svg .node text),
  :global(.mermaid-svg-container svg .nodeLabel),
  :global(.mermaid-svg-container svg span.nodeLabel),
  :global(.mermaid-svg-container svg .label text),
  :global(.mermaid-svg-container svg text.actor),
  :global(.mermaid-svg-container svg .actor text),
  :global(.mermaid-svg-container svg .messageText),
  :global(.mermaid-svg-container svg .er.entityLabel),
  :global(.mermaid-svg-container svg .classText) { font-weight: 500; }
  :global(.mermaid-svg-container svg .node text),
  :global(.mermaid-svg-container svg .nodeLabel),
  :global(.mermaid-svg-container svg span.nodeLabel),
  :global(.mermaid-svg-container svg .label text) { color: #f8fafc !important; fill: #f8fafc !important; }
  :global(.mermaid-svg-container svg .edgeLabel rect) { fill: #090d16 !important; opacity: 0.95; }
  :global(.mermaid-svg-container svg .edgeLabel text),
  :global(.mermaid-svg-container svg .edgeLabel .nodeLabel),
  :global(.mermaid-svg-container svg .edgeLabel span) { fill: #f1f5f9 !important; color: #f1f5f9 !important; }
  :global(.mermaid-svg-container svg .cluster text) { fill: #38bdf8 !important; font-weight: 600; }
  :global(.contrast-light .mermaid-svg-container svg text),
  :global(.contrast-light .mermaid-svg-container svg .nodeLabel),
  :global(.contrast-light .mermaid-svg-container svg span.nodeLabel),
  :global(.contrast-light .mermaid-svg-container svg .label text) { fill: #0f172a !important; color: #0f172a !important; }
  :global(.contrast-light .mermaid-svg-container svg .edgeLabel rect) { fill: #ffffff !important; }
  :global(.contrast-light .mermaid-svg-container svg .edgeLabel text),
  :global(.contrast-light .mermaid-svg-container svg .edgeLabel .nodeLabel),
  :global(.contrast-light .mermaid-svg-container svg .edgeLabel span) { fill: #0f172a !important; color: #0f172a !important; }
</style>
