<script lang="ts">
  import GitGraph from "lucide-svelte/icons/git-graph";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import ZoomIn from "lucide-svelte/icons/zoom-in";
  import ZoomOut from "lucide-svelte/icons/zoom-out";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import ArrowUp from "lucide-svelte/icons/arrow-up";
  import ArrowDown from "lucide-svelte/icons/arrow-down";
  import ArrowLeft from "lucide-svelte/icons/arrow-left";
  import ArrowRight from "lucide-svelte/icons/arrow-right";
  import Maximize2 from "lucide-svelte/icons/maximize-2";
  import Minimize2 from "lucide-svelte/icons/minimize-2";
  import X from "lucide-svelte/icons/x";
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
  let isFullscreen = $state(false);

  // Pan & Zoom Reactive States
  let scale = $state(1);
  let panX = $state(0);
  let panY = $state(0);
  let isDragging = $state(false);
  let dragStartX = 0;
  let dragStartY = 0;
  let initialPanX = 0;
  let initialPanY = 0;

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

  $effect(() => {
    if (typeof window === "undefined") return;
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape" && isFullscreen) isFullscreen = false;
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  });

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(code);
      isCopied = true;
      setTimeout(() => (isCopied = false), 1800);
    } catch {}
  }

  function zoomBy(factor: number) {
    scale = Math.min(Math.max(scale * factor, 0.25), 4.5);
  }

  function panBy(dx: number, dy: number) {
    panX += dx;
    panY += dy;
  }

  function resetView() {
    scale = 1;
    panX = 0;
    panY = 0;
  }

  function handlePointerDown(e: PointerEvent) {
    if (e.button !== 0) return; // primary click only
    isDragging = true;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    initialPanX = panX;
    initialPanY = panY;
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
    try {
      (e.currentTarget as HTMLElement)?.releasePointerCapture?.(e.pointerId);
    } catch {}
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.1 : 0.9;
    zoomBy(factor);
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
      class="relative w-full h-[360px] overflow-hidden bg-slate-950/80 {isDragging ? 'cursor-grabbing' : 'cursor-grab'} touch-none"
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
      <div class="absolute bottom-2.5 right-2.5 flex items-center gap-1 z-10 pointer-events-auto">
        <!-- Directional D-Pad (Up, Down, Left, Right) -->
        <div class="grid grid-cols-3 gap-0.5 p-1 rounded-md bg-slate-900/85 backdrop-blur-md border border-white/10 shadow-lg text-slate-300">
          <div></div>
          <button type="button" onclick={() => panBy(0, 60)} class="p-1 rounded hover:bg-white/15" title="Pan Atas"><ArrowUp class="h-3 w-3" /></button>
          <div></div>
          <button type="button" onclick={() => panBy(60, 0)} class="p-1 rounded hover:bg-white/15" title="Pan Kiri"><ArrowLeft class="h-3 w-3" /></button>
          <button type="button" onclick={resetView} class="p-1 rounded hover:bg-white/15 font-mono text-[9px] text-center" title="Reset">{Math.round(scale * 100)}%</button>
          <button type="button" onclick={() => panBy(-60, 0)} class="p-1 rounded hover:bg-white/15" title="Pan Kanan"><ArrowRight class="h-3 w-3" /></button>
          <div></div>
          <button type="button" onclick={() => panBy(0, -60)} class="p-1 rounded hover:bg-white/15" title="Pan Bawah"><ArrowDown class="h-3 w-3" /></button>
          <div></div>
        </div>

        <!-- Zoom Controls -->
        <div class="flex flex-col gap-0.5 p-1 rounded-md bg-slate-900/85 backdrop-blur-md border border-white/10 shadow-lg text-slate-300">
          <button type="button" onclick={() => zoomBy(1.2)} class="p-1 rounded hover:bg-white/15" title="Zoom In"><ZoomIn class="h-3 w-3" /></button>
          <button type="button" onclick={() => zoomBy(0.8)} class="p-1 rounded hover:bg-white/15" title="Zoom Out"><ZoomOut class="h-3 w-3" /></button>
          <button type="button" onclick={resetView} class="p-1 rounded hover:bg-white/15" title="Reset View"><RotateCcw class="h-2.5 w-2.5" /></button>
        </div>
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
      <button
        type="button"
        onclick={() => (isFullscreen = false)}
        class="p-1.5 rounded hover:bg-white/10 text-slate-300 hover:text-white transition-colors"
        title="Tutup Fullscreen (Esc)"
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <!-- Fullscreen Interactive Canvas -->
    <div
      class="relative flex-1 w-full rounded-lg border border-white/10 bg-slate-900/60 overflow-hidden shadow-2xl {isDragging ? 'cursor-grabbing' : 'cursor-grab'} touch-none"
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
      <div class="absolute bottom-4 right-4 flex items-center gap-1.5 z-10 pointer-events-auto">
        <div class="grid grid-cols-3 gap-0.5 p-1.5 rounded-lg bg-slate-900/90 backdrop-blur-md border border-white/15 shadow-xl text-slate-200">
          <div></div>
          <button type="button" onclick={() => panBy(0, 100)} class="p-1.5 rounded hover:bg-white/15" title="Pan Atas"><ArrowUp class="h-3.5 w-3.5" /></button>
          <div></div>
          <button type="button" onclick={() => panBy(100, 0)} class="p-1.5 rounded hover:bg-white/15" title="Pan Kiri"><ArrowLeft class="h-3.5 w-3.5" /></button>
          <button type="button" onclick={resetView} class="p-1 rounded hover:bg-white/15 font-mono text-[10px] text-center" title="Reset">{Math.round(scale * 100)}%</button>
          <button type="button" onclick={() => panBy(-100, 0)} class="p-1.5 rounded hover:bg-white/15" title="Pan Kanan"><ArrowRight class="h-3.5 w-3.5" /></button>
          <div></div>
          <button type="button" onclick={() => panBy(0, -100)} class="p-1.5 rounded hover:bg-white/15" title="Pan Bawah"><ArrowDown class="h-3.5 w-3.5" /></button>
          <div></div>
        </div>
        <div class="flex flex-col gap-0.5 p-1.5 rounded-lg bg-slate-900/90 backdrop-blur-md border border-white/15 shadow-xl text-slate-200">
          <button type="button" onclick={() => zoomBy(1.25)} class="p-1.5 rounded hover:bg-white/15" title="Zoom In"><ZoomIn class="h-3.5 w-3.5" /></button>
          <button type="button" onclick={() => zoomBy(0.8)} class="p-1.5 rounded hover:bg-white/15" title="Zoom Out"><ZoomOut class="h-3.5 w-3.5" /></button>
          <button type="button" onclick={resetView} class="p-1.5 rounded hover:bg-white/15" title="Reset View"><RotateCcw class="h-3 w-3" /></button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(.mermaid-svg-container svg) {
    max-width: none !important;
    height: auto;
    display: block;
    margin: 0 auto;
  }
</style>
